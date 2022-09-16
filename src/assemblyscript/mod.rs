mod common;
mod function;
mod header;
mod r#struct;
mod tuple;
mod union;

use std::io::Write;

use common::*;

use super::*;
use crate::astype::*;
use crate::error::*;
use crate::pretty_writer::PrettyWriter;

pub struct AssemblyScriptGenerator {
    module_name: Option<String>,
    typescript_mode: bool,
}

impl AssemblyScriptGenerator {
    pub fn new(module_name: Option<String>, typescript_mode: bool) -> Self {
        AssemblyScriptGenerator { module_name, typescript_mode: typescript_mode }
    }
}
impl<T: Write> Generator<T> for AssemblyScriptGenerator {
    fn generate(
        &self,
        writer: &mut T,
        doc_witx: witx::Document,
        options: &Options,
    ) -> Result<(), Error> {
        for module_witx in doc_witx.modules() {
            let mut w = PrettyWriter::new(&mut *writer, "    ");
            let module_name = match &self.module_name {
                None => module_witx.name.as_str().to_string(),
                Some(module_name) => module_name.to_string(),
            };
            let _module_id = module_witx.name.as_str();
            let skip_imports = options.skip_imports;

            if !options.skip_header {
                Self::header(&self, &mut w)?;
            }

            let module_title_comments = format!(
                "---------------------- Module: [{}] ----------------------",
                module_name
            );
            Self::write_comments(&mut w, &module_title_comments)?;
            w.eob()?;
            for type_ in doc_witx.typenames() {
                //if skip_imports && &type_.module != module_id {
                if skip_imports {
                    continue;
                }
                let constants_for_type: Vec<_> = doc_witx
                    .constants()
                    .into_iter()
                    .filter_map(|x| {
                        if x.ty == type_.name {
                            let docs = x.docs.as_str().to_string();
                            Some(ASConstant {
                                name: x.name.as_str().to_string(),
                                docs: docs,
                                value: x.value,
                            })
                        } else {
                            None
                        }
                    })
                    .collect();
                Self::define_type(&self, &mut w, type_.as_ref(), &constants_for_type)?;
            }

            let mut gen_module_name = module_name.clone();
            if options.async_mode {
                gen_module_name = format!("{}_async",gen_module_name);
            }

            if options.export_mode {
                w.write_line(format!("export interface {} {{", gen_module_name.as_type()))?;
                w = w.new_block();
            }

            for func in module_witx.funcs() {
                Self::define_func(&self, &mut w, &module_name, func.as_ref(), options)?;
            }

            if options.export_mode {
                w.write("}")?.eob()?;
                w.eol()?;
            }

            if options.export_mode {
                let mut errorWrapperParameter = "";
                if options.error_wrapper {
                    errorWrapperParameter = "errorHandler1: (err: any) => number, ";
                }
                w.write(format!("export function add{}ToImports(imports: any, obj: {}, {}get_export: (name: string) => WebAssembly.ExportValue): void {{", module_name.as_type(), gen_module_name.as_type(), errorWrapperParameter))?;
                w.eol()?;
                w = w.new_block();

                w.write_line(format!("if (!(\"{}\" in imports)) imports[\"{}\"] = {{}};", module_name, module_name))?;
                for func in module_witx.funcs() {
                    Self::define_export_func_wrapper(&self, &mut w, &module_name, func.as_ref(), options)?;
                }
                w.write("}")?.eob()?;
            }
        }
        Ok(())
    }
}

impl AssemblyScriptGenerator {
    fn write_docs<T: Write>(w: &mut PrettyWriter<T>, docs: &str) -> Result<(), Error> {
        if docs.is_empty() {
            return Ok(());
        }
        w.write_line("/**")?;
        for docs_line in docs.lines() {
            if docs_line.is_empty() {
                w.write_line(" *")?;
            } else {
                w.write_line(format!(" * {}", docs_line))?;
            }
        }
        w.write_line(" */")?;
        Ok(())
    }

    fn write_comments<T: Write>(w: &mut PrettyWriter<T>, docs: &str) -> Result<(), Error> {
        if docs.is_empty() {
            return Ok(());
        }
        w.write_line("/*")?;
        for docs_line in docs.lines() {
            if docs_line.is_empty() {
                w.write_line(" *")?;
            } else {
                w.write_line(format!(" * {}", docs_line))?;
            }
        }
        w.write_line(" */")?;
        Ok(())
    }

    fn define_as_alias<T: Write>(
        &self,
        w: &mut PrettyWriter<T>,
        name: &str,
        other_type: &ASType,
    ) -> Result<(), Error> {
        if self.typescript_mode {
            w.write_line(format!(
                "export const {} = {};",
                name.as_type(),
                other_type.as_lang()
            ))?;
            w.write_line(format!(
                "export type {} = {};",
                name.as_type(),
                other_type.as_lang()
            ))?;
        } else {
            w.write_line(format!(
                "export type {} = {};",
                name.as_type(),
                other_type.as_lang()
            ))?;
        }
        Ok(())
    }

    fn define_as_atom<T: Write>(
        &self,
        w: &mut PrettyWriter<T>,
        name: &str,
        type_: &ASType,
    ) -> Result<(), Error> {
        if self.typescript_mode {
            w.write_line(format!(
                "export const {} = {};",
                name.as_type(),
                type_.as_lang_with_parent(Some(type_))
            ))?;
            w.write_line(format!(
                "export type {} = TargetType<typeof {}>;",
                name.as_type(),
                type_.as_lang_with_parent(Some(type_))
            ))?;
        } else {
            w.write_line(format!(
                "export type {} = {};",
                name.as_type(),
                type_.as_lang()
            ))?;
        }
        Ok(())
    }

    fn define_as_enum<T: Write>(
        &self,
        w: &mut PrettyWriter<T>,
        name: &str,
        enum_: &ASEnum,
    ) -> Result<(), Error> {
        if self.typescript_mode {
            let repr = enum_.repr.as_ref();
            w.write_line(format!(
                "export const {} = enumer<{}>({});",
                name.as_type(),
                name.as_namespace(),
                repr.as_lang()
            ))?;
            w.write_line(format!(
                "export type {} = TargetType<typeof {}>;",
                name.as_type(),
                name.as_type()
            ))?;
            w.eob()?;
            w.write_line(format!("export const enum {} {{", name.as_namespace()))?;
            {
                let mut w = w.new_block();
                for choice in &enum_.choices {
                    let docs = &choice.docs;
                    if !docs.is_empty() {
                        Self::write_docs(&mut w, docs)?;
                    }
                    w.write_line(format!(
                        "{} = {},",
                        choice.name.as_const(),
                        choice.value
                    ))?;
                }
            }
            w.write_line("}")?;
        } else {
            let repr = enum_.repr.as_ref();
            w.write_line(format!(
                "export type {} = {};",
                name.as_type(),
                repr.as_lang()
            ))?;
            w.eob()?;
            w.write_line(format!("export namespace {} {{", name.as_namespace()))?;
            {
                let mut w = w.new_block();
                for choice in &enum_.choices {
                    w.write_line(format!(
                        "export const {}: {} = {};",
                        choice.name.as_const(),
                        name.as_type(),
                        choice.value
                    ))?;
                }
            }
            w.write_line("}")?;
        }
        Ok(())
    }

    fn define_as_constants<T: Write>(
        &self,
        w: &mut PrettyWriter<T>,
        name: &str,
        constants: &ASConstants,
    ) -> Result<(), Error> {
        let repr = constants.repr.as_ref();
        if self.typescript_mode {
            w.write_line(format!(
                "export const {} = {};",
                name.as_type(),
                repr.as_lang()
            ))?;
            w.write_line(format!(
                "export type {} = TargetType<typeof {}>;",
                name.as_type(),
                repr.as_lang()
            ))?;
            w.eob()?;
            Self::define_constants_for_type(&self, w, name, Some(repr), &constants.constants)?;
        } else {
            w.write_line(format!(
                "export type {} = {};",
                name.as_type(),
                repr.as_lang()
            ))?;
            w.eob()?;
            Self::define_constants_for_type(&self, w, name, Some(repr), &constants.constants)?;
        }
        Ok(())
    }

    fn define_as_type<T: Write>(
        &self,
        w: &mut PrettyWriter<T>,
        name: &str,
        type_: &ASType,
    ) -> Result<(), Error> {
        match type_ {
            ASType::Alias(_)
            | ASType::Bool
            | ASType::Char8
            | ASType::Char32
            | ASType::F32
            | ASType::F64
            | ASType::U8
            | ASType::U16
            | ASType::U32
            | ASType::U64
            | ASType::S8
            | ASType::S16
            | ASType::S32
            | ASType::S64
            | ASType::USize
            | ASType::Handle(_)
            | ASType::Slice(_)
            | ASType::String(_)
            | ASType::ReadBuffer(_)
            | ASType::WriteBuffer(_) => Self::define_as_atom(&self, w, name, type_)?,
            ASType::Enum(enum_) => Self::define_as_enum(&self, w, name, enum_)?,
            ASType::Union(union_) => Self::define_as_union(&self, w, name, union_)?,
            ASType::Constants(constants) => Self::define_as_constants(&self, w, name, constants)?,
            ASType::Tuple(members) => Self::define_as_tuple(w, name, members)?,
            ASType::Struct(members) => Self::define_as_struct(&self, w, type_, name, members)?,
            _ => {
                dbg!(type_);
                unimplemented!();
            }
        }
        Ok(())
    }

    fn define_constants_for_type<T: Write>(
        &self,
        w: &mut PrettyWriter<T>,
        type_name: &str,
        parent_type: Option<&ASType>,
        constants: &[ASConstant],
    ) -> Result<(), Error> {
        if constants.is_empty() {
            return Ok(());
        }
        if self.typescript_mode {
            w.write_line(format!("export const {} = {{", type_name.as_namespace()))?;
            {
                let mut w = w.new_block();
                let mut hex = false;
                let mut bigint = false;
                let mut single_bits: usize = 0;
                match parent_type {
                    Some(par_type) => {
                        match par_type {
                            ASType::U64 => {
                                bigint = true;
                            }
                            ASType::S64 => {
                                bigint = true;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
                for constant in constants {
                    if constant.value > 0xffff {
                        hex = true;
                    }
                    if constant.value.count_ones() == 1 {
                        single_bits += 1;
                    }
                }
                if constants.len() > 2 && single_bits == constants.len() {
                    hex = true;
                }
                for constant in constants {
                    let value_s = if hex {
                        if bigint {
                            format!("0x{:x}n", constant.value)
                        } else {
                            format!("0x{:x}", constant.value)
                        }
                    } else {
                        if bigint {
                            format!("{}n", constant.value)
                        } else {
                            format!("{}", constant.value)
                        }
                    };
                    let docs = constant.docs.as_str();
                    if !docs.is_empty() {
                        Self::write_docs(&mut w, docs)?;
                    }
                    w.write_line(format!(
                        "{}: {},",
                        constant.name.as_const(),
                        value_s
                    ))?;
                }
            }
            w.write_line("}")?;
            w.eob()?;
        } else {
            w.write_line(format!("export namespace {} {{", type_name.as_namespace()))?;
            {
                let mut w = w.new_block();
                let mut hex = false;
                let mut single_bits: usize = 0;
                for constant in constants {
                    if constant.value > 0xffff {
                        hex = true;
                    }
                    if constant.value.count_ones() == 1 {
                        single_bits += 1;
                    }
                }
                if constants.len() > 2 && single_bits == constants.len() {
                    hex = true;
                }
                for constant in constants {
                    let value_s = if hex {
                        format!("0x{:x}", constant.value)
                    } else {
                        format!("{}", constant.value)
                    };
                    w.write_line(format!(
                        "export const {}: {} = {};",
                        constant.name.as_const(),
                        type_name.as_type(),
                        value_s
                    ))?;
                }
            }
            w.write_line("}")?;
            w.eob()?;
        }
        Ok(())
    }

    fn define_type<T: Write>(
        &self,
        w: &mut PrettyWriter<T>,
        type_witx: &witx::NamedType,
        constants: &[ASConstant],
    ) -> Result<(), Error> {
        let docs = &type_witx.docs;
        if !docs.is_empty() {
            Self::write_docs(w, docs)?;
        }
        let type_name = type_witx.name.as_str();
        let tref = &type_witx.tref;
        match tref {
            witx::TypeRef::Name(other_type) => {
                Self::define_as_alias(&self, w, type_name, &ASType::from(&other_type.tref))?
            }
            witx::TypeRef::Value(type_witx) => {
                let t = ASType::from(type_witx.as_ref());
                Self::define_as_type(&self, w, type_name, &t)?
            }
        }
        w.eob()?;
        Self::define_constants_for_type(&self, w, type_name, None, constants)?;
        Ok(())
    }
}
