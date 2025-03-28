use std::borrow::Cow;
use std::ffi::CString;
use std::path::Path;

use crate::fluent_generated as fluent;
use rustc_data_structures::small_c_str::SmallCStr;
use rustc_errors::{
    DiagnosticBuilder, EmissionGuarantee, ErrorGuaranteed, FatalError, Handler, IntoDiagnostic,
};
use rustc_macros::{Diagnostic, Subdiagnostic};
use rustc_span::Span;

#[derive(Diagnostic)]
#[diag(codegen_llvm_unknown_ctarget_feature_prefix)]
#[note]
pub(crate) struct UnknownCTargetFeaturePrefix<'a> {
    pub feature: &'a str,
}

#[derive(Diagnostic)]
#[diag(codegen_llvm_unknown_ctarget_feature)]
#[note]
pub(crate) struct UnknownCTargetFeature<'a> {
    pub feature: &'a str,
    #[subdiagnostic]
    pub rust_feature: PossibleFeature<'a>,
}

#[derive(Diagnostic)]
#[diag(codegen_llvm_unstable_ctarget_feature)]
#[note]
pub(crate) struct UnstableCTargetFeature<'a> {
    pub feature: &'a str,
}

#[derive(Subdiagnostic)]
pub(crate) enum PossibleFeature<'a> {
    #[help(codegen_llvm_possible_feature)]
    Some { rust_feature: &'a str },
    #[help(codegen_llvm_consider_filing_feature_request)]
    None,
}

#[derive(Diagnostic)]
#[diag(codegen_llvm_error_creating_import_library)]
pub(crate) struct ErrorCreatingImportLibrary<'a> {
    pub lib_name: &'a str,
    pub error: String,
}

#[derive(Diagnostic)]
#[diag(codegen_llvm_symbol_already_defined)]
pub(crate) struct SymbolAlreadyDefined<'a> {
    #[primary_span]
    pub span: Span,
    pub symbol_name: &'a str,
}

#[derive(Diagnostic)]
#[diag(codegen_llvm_invalid_minimum_alignment_not_power_of_two)]
pub(crate) struct InvalidMinimumAlignmentNotPowerOfTwo {
    pub align: u64,
}

#[derive(Diagnostic)]
#[diag(codegen_llvm_invalid_minimum_alignment_too_large)]
pub(crate) struct InvalidMinimumAlignmentTooLarge {
    pub align: u64,
}

#[derive(Diagnostic)]
#[diag(codegen_llvm_sanitizer_memtag_requires_mte)]
pub(crate) struct SanitizerMemtagRequiresMte;

#[derive(Diagnostic)]
#[diag(codegen_llvm_error_writing_def_file)]
pub(crate) struct ErrorWritingDEFFile {
    pub error: std::io::Error,
}

#[derive(Diagnostic)]
#[diag(codegen_llvm_error_calling_dlltool)]
pub(crate) struct ErrorCallingDllTool<'a> {
    pub dlltool_path: Cow<'a, str>,
    pub error: std::io::Error,
}

#[derive(Diagnostic)]
#[diag(codegen_llvm_dlltool_fail_import_library)]
pub(crate) struct DlltoolFailImportLibrary<'a> {
    pub dlltool_path: Cow<'a, str>,
    pub dlltool_args: String,
    pub stdout: Cow<'a, str>,
    pub stderr: Cow<'a, str>,
}

#[derive(Diagnostic)]
#[diag(codegen_llvm_dynamic_linking_with_lto)]
#[note]
pub(crate) struct DynamicLinkingWithLTO;

pub(crate) struct ParseTargetMachineConfig<'a>(pub LlvmError<'a>);

impl IntoDiagnostic<'_, FatalError> for ParseTargetMachineConfig<'_> {
    fn into_diagnostic(self, handler: &'_ Handler) -> DiagnosticBuilder<'_, FatalError> {
        let diag: DiagnosticBuilder<'_, FatalError> = self.0.into_diagnostic(handler);
        let (message, _) = diag.styled_message().first().expect("`LlvmError` with no message");
        let message = handler.eagerly_translate_to_string(message.clone(), diag.args());

        let mut diag =
            handler.struct_almost_fatal(fluent::codegen_llvm_parse_target_machine_config);
        diag.set_arg("error", message);
        diag
    }
}

pub(crate) struct TargetFeatureDisableOrEnable<'a> {
    pub features: &'a [&'a str],
    pub span: Option<Span>,
    pub missing_features: Option<MissingFeatures>,
}

#[derive(Subdiagnostic)]
#[help(codegen_llvm_missing_features)]
pub(crate) struct MissingFeatures;

impl IntoDiagnostic<'_, ErrorGuaranteed> for TargetFeatureDisableOrEnable<'_> {
    fn into_diagnostic(self, handler: &'_ Handler) -> DiagnosticBuilder<'_, ErrorGuaranteed> {
        let mut diag = handler.struct_err(fluent::codegen_llvm_target_feature_disable_or_enable);
        if let Some(span) = self.span {
            diag.set_span(span);
        };
        if let Some(missing_features) = self.missing_features {
            diag.subdiagnostic(missing_features);
        }
        diag.set_arg("features", self.features.join(", "));
        diag
    }
}

#[derive(Diagnostic)]
#[diag(codegen_llvm_lto_disallowed)]
pub(crate) struct LtoDisallowed;

#[derive(Diagnostic)]
#[diag(codegen_llvm_lto_dylib)]
pub(crate) struct LtoDylib;

#[derive(Diagnostic)]
#[diag(codegen_llvm_lto_proc_macro)]
pub(crate) struct LtoProcMacro;

#[derive(Diagnostic)]
#[diag(codegen_llvm_lto_bitcode_from_rlib)]
pub(crate) struct LtoBitcodeFromRlib {
    pub llvm_err: String,
}

#[derive(Diagnostic)]
pub enum LlvmError<'a> {
    #[diag(codegen_llvm_write_output)]
    WriteOutput { path: &'a Path },
    #[diag(codegen_llvm_target_machine)]
    CreateTargetMachine { triple: SmallCStr },
    #[diag(codegen_llvm_run_passes)]
    RunLlvmPasses,
    #[diag(codegen_llvm_serialize_module)]
    SerializeModule { name: &'a str },
    #[diag(codegen_llvm_write_ir)]
    WriteIr { path: &'a Path },
    #[diag(codegen_llvm_prepare_thin_lto_context)]
    PrepareThinLtoContext,
    #[diag(codegen_llvm_load_bitcode)]
    LoadBitcode { name: CString },
    #[diag(codegen_llvm_write_thinlto_key)]
    WriteThinLtoKey { err: std::io::Error },
    #[diag(codegen_llvm_multiple_source_dicompileunit)]
    MultipleSourceDiCompileUnit,
    #[diag(codegen_llvm_prepare_thin_lto_module)]
    PrepareThinLtoModule,
    #[diag(codegen_llvm_parse_bitcode)]
    ParseBitcode,
}

pub(crate) struct WithLlvmError<'a>(pub LlvmError<'a>, pub String);

impl<G: EmissionGuarantee> IntoDiagnostic<'_, G> for WithLlvmError<'_> {
    fn into_diagnostic(self, handler: &'_ Handler) -> DiagnosticBuilder<'_, G> {
        use LlvmError::*;
        let msg_with_llvm_err = match &self.0 {
            WriteOutput { .. } => fluent::codegen_llvm_write_output_with_llvm_err,
            CreateTargetMachine { .. } => fluent::codegen_llvm_target_machine_with_llvm_err,
            RunLlvmPasses => fluent::codegen_llvm_run_passes_with_llvm_err,
            SerializeModule { .. } => fluent::codegen_llvm_serialize_module_with_llvm_err,
            WriteIr { .. } => fluent::codegen_llvm_write_ir_with_llvm_err,
            PrepareThinLtoContext => fluent::codegen_llvm_prepare_thin_lto_context_with_llvm_err,
            LoadBitcode { .. } => fluent::codegen_llvm_load_bitcode_with_llvm_err,
            WriteThinLtoKey { .. } => fluent::codegen_llvm_write_thinlto_key_with_llvm_err,
            MultipleSourceDiCompileUnit => {
                fluent::codegen_llvm_multiple_source_dicompileunit_with_llvm_err
            }
            PrepareThinLtoModule => fluent::codegen_llvm_prepare_thin_lto_module_with_llvm_err,
            ParseBitcode => fluent::codegen_llvm_parse_bitcode_with_llvm_err,
        };
        let mut diag = self.0.into_diagnostic(handler);
        diag.set_primary_message(msg_with_llvm_err);
        diag.set_arg("llvm_err", self.1);
        diag
    }
}

#[derive(Diagnostic)]
#[diag(codegen_llvm_from_llvm_optimization_diag)]
pub(crate) struct FromLlvmOptimizationDiag<'a> {
    pub filename: &'a str,
    pub line: std::ffi::c_uint,
    pub column: std::ffi::c_uint,
    pub pass_name: &'a str,
    pub kind: &'a str,
    pub message: &'a str,
}

#[derive(Diagnostic)]
#[diag(codegen_llvm_from_llvm_diag)]
pub(crate) struct FromLlvmDiag {
    pub message: String,
}

#[derive(Diagnostic)]
#[diag(codegen_llvm_write_bytecode)]
pub(crate) struct WriteBytecode<'a> {
    pub path: &'a Path,
    pub err: std::io::Error,
}

#[derive(Diagnostic)]
#[diag(codegen_llvm_copy_bitcode)]
pub(crate) struct CopyBitcode {
    pub err: std::io::Error,
}

#[derive(Diagnostic)]
#[diag(codegen_llvm_unknown_debuginfo_compression)]
pub struct UnknownCompression {
    pub algorithm: &'static str,
}
