use std::{io::Write, rc::Rc};

use super::*;

impl AssemblyScriptGenerator {

    pub fn get_func_params_processed<T: Write>(
        &self,
        _w: &mut PrettyWriter<T>,
        _module_name: &str,
        func_witx: &witx::InterfaceFunc,
        _options: &Options,
    ) -> Result<(Vec<ASTypeDecomposed>,Vec<(String, Rc<ASType>)>,Option<ASResult> ), Error> {
        let mut ok_type = ASType::Void;
        let params_witx = &func_witx.params;
        let mut params = vec![];
        for param_witx in params_witx {
            let param_name = param_witx.name.as_str();
            let param_type = ASType::from(&param_witx.tref);
            params.push((param_name.to_string(), param_type));
        }

        let results_witx = &func_witx.results;
        //println!("assemblyscript funcName: {}",func_witx.name.as_str());
        //assert_eq!(results_witx.len(), 1);
        let mut params_decomposed = vec![];
        let mut results = vec![];
        let mut o_result: Option<ASResult>  = None;

        if results_witx.len() > 0{
            let result_witx = &results_witx[0];
            let result = ASType::from(&result_witx.tref);
            let result = match result {
                ASType::Result(result) => result,
                _ => unreachable!(),
            };
            ok_type = result.to_owned().ok_type.as_ref().to_owned();
            o_result = Some(result);
        }

        for param in &params {
            let mut decomposed = param.1.decompose(&param.0, false);
            params_decomposed.append(&mut decomposed);
        }

        // A tuple in a result is expanded into additional parameters, transformed to
        // pointers
        if let ASType::Tuple(tuple_members) = ok_type.leaf() {
            for (i, tuple_member) in tuple_members.iter().enumerate() {
                let name = format!("result{}_ptr", i);
                results.push((name, tuple_member.type_.clone()));
            }
        } else {
            let name = "result_ptr";
            results.push((name.to_string(), Rc::new(ok_type)));
        }
        for result in &results {
            let mut decomposed = result.1.decompose(&result.0, true);
            params_decomposed.append(&mut decomposed);
        }
        
        Ok((params_decomposed, results, o_result))
    }



    pub fn define_func<T: Write>(
        &self,
        w: &mut PrettyWriter<T>,
        module_name: &str,
        func_witx: &witx::InterfaceFunc,
        options: &Options,
    ) -> Result<(), Error> {
        assert_eq!(func_witx.abi, witx::Abi::Preview1);
        let name = func_witx.name.as_str().to_string();
        let mut no_result = true;
        let (params_decomposed, results, o_result) = Self::get_func_params_processed(self, w, module_name, func_witx, options)?;
        match o_result {
            Some(_) => { 
                no_result = false;
            }
            None => {
                no_result = true;
            }
        }
        let docs = &func_witx.docs;
        if !docs.is_empty() {
            Self::write_docs(w, docs)?;
        }

        if self.typescript_mode {
            if options.export_mode {
                w.indent()?
                    .write(format!("{}(", name.as_fn()))?;
            } else {
                w.write(format!("export declare function {}(", name.as_fn()))?;
            }
        } else {
            w.write_line("// @ts-ignore: decorator")?
                .write_line("@unsafe")?
                .write_line("// @ts-ignore: decorator")?
                .write_line(format!("@external(\"{}\", \"{}\")", module_name, name))?
                .indent()?
                .write(format!("export declare function {}(", name.as_fn()))?;
        }

        if !params_decomposed.is_empty() || !results.is_empty() {
            w.eol()?;
        }
        for (i, param) in params_decomposed.iter().enumerate() {
            let eol = if i + 1 == params_decomposed.len() {
                ""
            } else {
                ","
            };
            w.write_line_continued(format!(
                "{}: {}{}",
                param.name.as_var(),
                param.type_.as_lang(),
                eol
            ))?;
        }

        if options.async_mode {
            match o_result {
                Some(result) => {
                    w.write_line(format!("): Promise<{}>;", result.error_type.as_lang()))?;
                    w.eob()?;
                }
                None => {
                    w.write_line(format!("): Promise<void>;"))?;
                    w.eob()?;
                }
            }
        } else {
            match o_result {
                Some(result) => {
                    w.write_line(format!("): {};", result.error_type.as_lang()))?;
                    w.eob()?;
                }
                None => {
                    w.write_line(format!(");"))?;
                    w.eob()?;
                }
            }
        }

        //let signature_witx = func_witx.wasm_signature(witx::CallMode::DefinedImport);
        let signature_witx = func_witx.wasm_signature();
        let signature_witx_params = signature_witx.0;
        let signature_witx_results = signature_witx.1;

        let params_count_witx = signature_witx_params.len() + signature_witx_results.len();
        if no_result {
            assert_eq!(params_count_witx, params_decomposed.len() + 0);
        } else {
            assert_eq!(params_count_witx, params_decomposed.len() + 1);
        }

        Ok(())
    }

    pub fn define_export_func_wrapper<T: Write>(
        &self,
        w: &mut PrettyWriter<T>,
        module_name: &str,
        func_witx: &witx::InterfaceFunc,
        options: &Options,
    ) -> Result<(), Error> {
        assert_eq!(func_witx.abi, witx::Abi::Preview1);
        let name = func_witx.name.as_str().to_string();

        let (params_decomposed, results, o_result) = Self::get_func_params_processed(self, w, module_name, func_witx, options)?;

        /* old impl 
        if self.typescript_mode {
            if options.export_mode {
                w.write_line(format!("imports[\"{}\"][\"{}\"] = obj.{}", module_name, func_witx.name.as_str().to_string(),func_witx.name.as_fn()))?;
            } else {
                unreachable!("Only supported for export");
            }
        } else {
            unreachable!("Only supported for typescript");
        }
        */

        if self.typescript_mode {
            if options.export_mode {
                let mut async_modifier = "";
                let mut await_keyword = "";
                let mut is_void_return: bool = false;
                match o_result {
                    Some(_) => is_void_return = false,
                    None => is_void_return = true,
                }
                if options.async_mode {
                    async_modifier = "async";
                    await_keyword = "await";
                }
                w.indent()?.write(format!("imports[\"{}\"][\"{}\"] = {} function (", module_name, func_witx.name.as_str().to_string(), async_modifier))?;
                for (i, param) in params_decomposed.iter().enumerate() {
                    let eol = if i + 1 == params_decomposed.len() {
                        ""
                    } else {
                        ","
                    };
                    w.write(format!(
                        "_{}: any{}",
                        param.name.as_var(),
                        eol
                    ))?;
                }
                w.write(") {")?;
                w.eol()?;
                let mut w = w.new_block();
                if options.error_handler {
                    w.indent()?.write("const errorHandler2 = handler.handleError")?.eol()?;
                    w.indent()?.write("try {")?.eol()?;
                    let mut w = w.new_block();
                }
                if is_void_return {
                    w.indent()?.indent()?.write(format!("{} obj.{}(", await_keyword, func_witx.name.as_fn()))?;
                } else {
                    w.indent()?.indent()?.write(format!("const ret = {} obj.{}(",await_keyword, func_witx.name.as_fn()))?;
                }
                for (i, param) in params_decomposed.iter().enumerate() {
                    let eol = if i + 1 == params_decomposed.len() {
                        ""
                    } else {
                        ","
                    };
                    w.write(format!(
                        "_{}{}",
                        param.name.as_var(),
                        eol
                    ))?;
                }
                w.write(");")?;
                w.eol()?;

                if options.error_handler {
                    w.indent()?.write_line("handler.checkAbort();")?;
                }
                if !is_void_return {
                    w.indent()?.write_line(format!("return ret;"))?;
                }
                if options.error_handler {
                    w.indent()?.write("} catch(err: any) {")?.eol()?;
                    if is_void_return {
                        w.write_line("const eRet = errorHandler2(err);")?;
                        w.write_line("console.log(\"handled error\", err, eRet);")?;
                    } else {
                        w.indent()?.write_line("return errorHandler2(err);")?;
                    }
                    w.indent()?.write("}")?.eol()?;
                    w.eob()?;
                }
                w.indent()?.write("}")?.eob()?;

            } else {
                unreachable!("Only supported for export");
            }
        } else {
            unreachable!("Only supported for typescript");
        }


        /* 
        if !params_decomposed.is_empty() || !results.is_empty() {
            w.eol()?;
        }
        for (i, param) in params_decomposed.iter().enumerate() {
            let eol = if i + 1 == params_decomposed.len() {
                ""
            } else {
                ","
            };
            w.write_line_continued(format!(
                "{}: {}{}",
                param.name.as_var(),
                param.type_.as_lang(),
                eol
            ))?;
        }

        if options.async_mode {
            match o_result {
                Some(result) => {
                    w.write_line(format!("): Promise<{}>;", result.error_type.as_lang()))?;
                    w.eob()?;
                }
                None => {
                    w.write_line(format!("): Promise<void>;"))?;
                    w.eob()?;
                }
            }
        } else {
            match o_result {
                Some(result) => {
                    w.write_line(format!("): {};", result.error_type.as_lang()))?;
                    w.eob()?;
                }
                None => {
                    w.write_line(format!(");"))?;
                    w.eob()?;
                }
            }
        }

        //let signature_witx = func_witx.wasm_signature(witx::CallMode::DefinedImport);
        let signature_witx = func_witx.wasm_signature();
        let signature_witx_params = signature_witx.0;
        let signature_witx_results = signature_witx.1;

        let params_count_witx = signature_witx_params.len() + signature_witx_results.len();
        assert_eq!(params_count_witx, params_decomposed.len() + 1);
        */
        Ok(())
    }

}
