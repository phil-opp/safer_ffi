use super::*;

pub
struct C;

impl HeaderLanguage for C {
    fn emit_docs (
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        indent: &'_ Indentation,
    ) -> io::Result<()>
    {
        mk_out!(indent, ctx.out());

        if docs.is_empty() {
            out!(("/** <No documentation available> */"));
            return Ok(());
        }

        out!(("/** \\brief"));
        for line in docs.iter().copied().map(str::trim) {
            let sep = if line.is_empty() { "" } else { "  " };
            out!((" *{sep}{line}"));
        }
        out!((" */"));

        Ok(())
    }

    fn emit_simple_enum (
        self: &'_ C,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        self_ty: &'_ dyn PhantomCType,
        backing_integer: Option<&dyn PhantomCType>,
        variants: &'_ [EnumVariant<'_>],
    ) -> io::Result<()>
    {
        let ref indent = Indentation::new(4 /* ctx.indent_width() */);
        mk_out!(indent, ctx.out());

        let ref intn_t =
            backing_integer.map(|it| it.name(self))
        ;

        self.emit_docs(ctx, docs, indent)?;

        let ref short_name = self_ty.short_name();
        let ref full_ty_name = self_ty.name(self);

        if let Some(intn_t) = intn_t {
            out!((
                "/** \\remark Has the same ABI as `{intn_t}` **/"
                "#ifdef DOXYGEN"
                "typedef"
                "#endif"
                "enum {short_name} {{"
            ));
        } else {
            out!(("typedef enum {short_name} {{"));
        }

        if let _ = indent.scope() {
            for v in variants {
                self.emit_docs(ctx, v.docs, indent)?;
                let variant_name = crate::utils::screaming_case(short_name, v.name) /* ctx.adjust_variant_name(
                    Language::C,
                    enum_name,
                    v.name,
                ) */;
                if let Some(value) = v.discriminant {
                    out!(("{variant_name} = {value:?},"));
                } else {
                    out!(("{variant_name},"));
                }
            }
        }

        if let Some(intn_t) = intn_t {
            out!((
                "}}"
                "#ifndef DOXYGEN"
                "; typedef {intn_t}"
                "#endif"
                "{full_ty_name};"
            ));
        } else {
            out!(("}} {full_ty_name};"));
        }

        out!("\n");
        Ok(())
    }

    fn emit_struct (
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        self_ty: &'_ dyn PhantomCType,
        fields: &'_ [StructField<'_>]
    ) -> io::Result<()>
    {
        let ref indent = Indentation::new(4 /* ctx.indent_width() */);
        mk_out!(indent, ctx.out());

        if self_ty.size() == 0 {
            panic!("C does not support zero-sized structs!")
        }
        let short_name = self_ty.short_name();
        let full_ty_name = self_ty.name(self);

        self.emit_docs(ctx, docs, indent)?;
        out!(("typedef struct {short_name} {{"));
        if let _ = indent.scope() {
            let ref mut first = true;
            for &StructField { docs, name, ty } in fields {
                if ty.size() == 0 && ty.align() > 1 {
                    panic!("Zero-sized fields must have an alignment of `1`");
                }
                if mem::take(first).not() {
                    out!("\n");
                }
                self.emit_docs(ctx, docs, indent)?;
                out!(
                    ("{};"),
                    ty.name_wrapping_var(self, name)
                );
            }
        }
        out!(("}} {full_ty_name};"));

        out!("\n");
        Ok(())
    }

    fn emit_function (
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        fname: &'_ str,
        arg_names: &'_ [FunctionArg<'_>],
        ret_ty: &'_ dyn PhantomCType,
    ) -> io::Result<()>
    {
        todo!()
    }
}