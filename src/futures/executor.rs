use super::*;

#[derive_ReprC(dyn)]
pub
trait FfiFutureExecutor : Send + Sync {
    fn dyn_spawn (
        self: &'_ Self,
        future: VirtualPtr<dyn 'static + FfiFuture>,
    ) -> VirtualPtr<dyn 'static + FfiFuture>
    ;

    fn dyn_spawn_blocking (
        self: &'_ Self,
        action: repr_c::Box<dyn 'static + Send + FnMut()>,
    ) -> VirtualPtr<dyn 'static + FfiFuture>
    ;

    fn dyn_block_on (
        self: &'_ Self,
        future: VirtualPtr<dyn '_ + FfiFuture>,
    )
    ;
}

match_! {([] [Send + Sync]) {( $([ $($SendSync:tt)* ])* ) => (
    $(
        impl VirtualPtr<dyn 'static + FfiFutureExecutor + $($SendSync)*> {
            pub
            fn spawn<R : 'static + Send> (
                self: &'_ Self,
                fut: impl 'static + Send + Future<Output = R>,
            ) -> impl Future<Output = R>
            {
                let (tx, rx) = ::futures::channel::oneshot::channel();
                let fut = self.dyn_spawn(
                    Box::new(async move {
                        tx.send(fut.await).ok();
                    })
                    .into()
                );
                async move {
                    fut.into_future().await;
                    rx  .await
                        .expect("\
                    executor dropped the `spawn`ed task before its completion\
                        ")
                }
            }

            pub
            fn spawn_blocking (
                self: &'_ Self,
                action: impl 'static + Send + FnOnce(),
            ) -> impl Future<Output = ()>
            {
                let mut action = Some(action);
                let action = move || {
                    action
                        .take()
                        .expect("\
                    executor called the `.spawn_blocking()` closure more than once\
                        ")
                        ()
                };
                self.dyn_spawn_blocking(Box::new(action).into())
                    .into_future()
            }

            pub
            fn block_on<R : Send> (
                self: &'_ Self,
                fut: impl Send + Future<Output = R>
            ) -> R
            {
                let mut ret = None;
                self.dyn_block_on(
                    Box::new(async {
                        ret = Some(fut.await);
                    })
                    .into()
                );
                ret.expect("`.dyn_block_on()` did not complete")
            }
        }

    )*
)}}

cfg_match! { feature = "tokio" => {
    impl FfiFutureExecutor for ::tokio::runtime::Handle {
        fn dyn_spawn (
            self: &'_ Self,
            future: VirtualPtr<dyn 'static + FfiFuture>,
        ) -> VirtualPtr<dyn 'static + FfiFuture>
        {
            let fut = self.spawn(future.into_future());
            let fut = async {
                fut .await
                    .unwrap_or_else(|caught_panic| {
                        ::std::panic::resume_unwind(caught_panic.into_panic())
                    })
            };
            Box::new(fut)
                .into()
        }

        fn dyn_spawn_blocking (
            self: &'_ Self,
            action: repr_c::Box<dyn 'static + Send + FnMut()>,
        ) -> VirtualPtr<dyn 'static + FfiFuture>
        {
            let fut = self.spawn_blocking(|| { action }.call());
            let fut = async {
                fut .await
                    .unwrap_or_else(|caught_panic| {
                        ::std::panic::resume_unwind(caught_panic.into_panic())
                    })
            };
            Box::new(fut)
                .into()
        }

        fn dyn_block_on (
            self: &'_ Self,
            future: VirtualPtr<dyn '_ + FfiFuture>,
        )
        {
            self.block_on(future.into_future())
        }
    }
}}

#[macro_export]
macro_rules! ffi_export_future_helpers {() => (
    const _: () = {
        use $crate::ඞ::std::{sync::Arc, task::Context, prelude::v1::*};

        #[ffi_export]
        fn rust_future_task_context_wake (
            task_context: &'static Context<'static>,
        )
        {
            task_context.waker().wake_by_ref()
        }

        #[ffi_export]
        fn rust_future_task_context_get_waker (
            task_context: &'static Context<'static>,
        ) -> $crate::prelude::repr_c::Arc<dyn 'static + Send + Sync + Fn()>
        {
            let waker = task_context.waker().clone();
            Arc::new(move || waker.wake_by_ref()).into()
        }
    };
)}