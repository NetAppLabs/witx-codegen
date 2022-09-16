use std::io::Write;

use super::*;

impl AssemblyScriptGenerator {

    pub fn define_as_struct<T: Write>(
        &self,
        w: &mut PrettyWriter<T>,
        parent_type: &ASType,
        name: &str,
        members: &[ASStructMember],
    ) -> Result<(), Error> {
        if self.typescript_mode {
            Self::define_as_struct_typescript(w, parent_type, name, members)
        } else {
            Self::define_as_struct_assemblyscript(w, parent_type, name, members)
        }
    }
    pub fn define_as_struct_assemblyscript<T: Write>(
        w: &mut PrettyWriter<T>,
        _parent_type: &ASType,
        name: &str,
        members: &[ASStructMember],
    ) -> Result<(), Error> {
        w.write_line("// @ts-ignore: decorator")?
            .write_line("@unmanaged")?
            .write_line(format!("export class {} {{", name.as_type()))?;
        {
            let mut w = w.new_block();
            for member in members {
                let member_type = member.type_.as_ref();
                w.write_line(format!(
                    "{}: {};",
                    member.name.as_var(),
                    member_type.as_lang()
                ))?;

                let pad_len = member.padding;
                for i in 0..(pad_len & 1) {
                    w.write_line(format!("private __pad8_{}: u8;", i))?;
                }
                for i in 0..(pad_len & 3) / 2 {
                    w.write_line(format!("private __pad16_{}: u16;", i))?;
                }
                for i in 0..(pad_len & 7) / 4 {
                    w.write_line(format!("private __pad32_{}: u32;", i))?;
                }
                for i in 0..pad_len / 8 {
                    w.write_line(format!("private __pad64_{}: u64;", i))?;
                }
            }
        }
        w.write_line("}")?.eob()?;
        Ok(())
    }

    pub fn define_as_struct_typescript<T: Write>(
        w: &mut PrettyWriter<T>,
        parent_type: &ASType,
        name: &str,
        members: &[ASStructMember],
    ) -> Result<(), Error> {
        w.write_line(format!("export const {} = struct({{", name.as_type()))?;
        {
            let mut w = w.new_block();
            for member in members {
                let member_type = member.type_.as_ref();
                w.write_line(format!(
                    "{}: {},",
                    member.name.as_var(),
                    member_type.as_lang_with_parent(Some(parent_type))
                ))?;
            }
        }
        w.write_line("});")?;
        w.write_line(format!("export type {} = TargetType<typeof {}>;", name.as_type(),name.as_type()))?
        .eob()?;
        Ok(())
    }
}
