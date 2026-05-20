// Generated automatically from agent_pb.ts schema. Do not edit manually.

#![allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    unused_mut,
    unused_variables,
    clippy::all
)]

use super::mod_impl::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum AppliedAgentChange_ChangeType {
    ChangeTypeUnspecified = 0,
    ChangeTypeCreated = 1,
    ChangeTypeModified = 2,
    ChangeTypeDeleted = 3,
}

impl Default for AppliedAgentChange_ChangeType {
    fn default() -> Self {
        Self::ChangeTypeUnspecified
    }
}

impl AppliedAgentChange_ChangeType {
    pub fn from_i32(val: i32) -> Self {
        match val {
            1 => Self::ChangeTypeCreated,
            2 => Self::ChangeTypeModified,
            3 => Self::ChangeTypeDeleted,
            _ => Self::ChangeTypeUnspecified,
        }
    }

    pub fn to_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum MouseButton {
    MouseButtonUnspecified = 0,
    MouseButtonLeft = 1,
    MouseButtonRight = 2,
    MouseButtonMiddle = 3,
    MouseButtonBack = 4,
    MouseButtonForward = 5,
}

impl Default for MouseButton {
    fn default() -> Self {
        Self::MouseButtonUnspecified
    }
}

impl MouseButton {
    pub fn from_i32(val: i32) -> Self {
        match val {
            1 => Self::MouseButtonLeft,
            2 => Self::MouseButtonRight,
            3 => Self::MouseButtonMiddle,
            4 => Self::MouseButtonBack,
            5 => Self::MouseButtonForward,
            _ => Self::MouseButtonUnspecified,
        }
    }

    pub fn to_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum ScrollDirection {
    ScrollDirectionUnspecified = 0,
    ScrollDirectionUp = 1,
    ScrollDirectionDown = 2,
    ScrollDirectionLeft = 3,
    ScrollDirectionRight = 4,
}

impl Default for ScrollDirection {
    fn default() -> Self {
        Self::ScrollDirectionUnspecified
    }
}

impl ScrollDirection {
    pub fn from_i32(val: i32) -> Self {
        match val {
            1 => Self::ScrollDirectionUp,
            2 => Self::ScrollDirectionDown,
            3 => Self::ScrollDirectionLeft,
            4 => Self::ScrollDirectionRight,
            _ => Self::ScrollDirectionUnspecified,
        }
    }

    pub fn to_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum CursorRuleSource {
    CursorRuleSourceUnspecified = 0,
    CursorRuleSourceTeam = 1,
    CursorRuleSourceUser = 2,
}

impl Default for CursorRuleSource {
    fn default() -> Self {
        Self::CursorRuleSourceUnspecified
    }
}

impl CursorRuleSource {
    pub fn from_i32(val: i32) -> Self {
        match val {
            1 => Self::CursorRuleSourceTeam,
            2 => Self::CursorRuleSourceUser,
            _ => Self::CursorRuleSourceUnspecified,
        }
    }

    pub fn to_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum DiagnosticSeverity {
    DiagnosticSeverityUnspecified = 0,
    DiagnosticSeverityError = 1,
    DiagnosticSeverityWarning = 2,
    DiagnosticSeverityInformation = 3,
    DiagnosticSeverityHint = 4,
}

impl Default for DiagnosticSeverity {
    fn default() -> Self {
        Self::DiagnosticSeverityUnspecified
    }
}

impl DiagnosticSeverity {
    pub fn from_i32(val: i32) -> Self {
        match val {
            1 => Self::DiagnosticSeverityError,
            2 => Self::DiagnosticSeverityWarning,
            3 => Self::DiagnosticSeverityInformation,
            4 => Self::DiagnosticSeverityHint,
            _ => Self::DiagnosticSeverityUnspecified,
        }
    }

    pub fn to_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum RecordingMode {
    RecordingModeUnspecified = 0,
    RecordingModeStartRecording = 1,
    RecordingModeSaveRecording = 2,
    RecordingModeDiscardRecording = 3,
}

impl Default for RecordingMode {
    fn default() -> Self {
        Self::RecordingModeUnspecified
    }
}

impl RecordingMode {
    pub fn from_i32(val: i32) -> Self {
        match val {
            1 => Self::RecordingModeStartRecording,
            2 => Self::RecordingModeSaveRecording,
            3 => Self::RecordingModeDiscardRecording,
            _ => Self::RecordingModeUnspecified,
        }
    }

    pub fn to_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum RequestedFilePathRejectedReason {
    RequestedFilePathRejectedReasonUnspecified = 0,
    RequestedFilePathRejectedReasonSlashesNotAllowed = 1,
}

impl Default for RequestedFilePathRejectedReason {
    fn default() -> Self {
        Self::RequestedFilePathRejectedReasonUnspecified
    }
}

impl RequestedFilePathRejectedReason {
    pub fn from_i32(val: i32) -> Self {
        match val {
            1 => Self::RequestedFilePathRejectedReasonSlashesNotAllowed,
            _ => Self::RequestedFilePathRejectedReasonUnspecified,
        }
    }

    pub fn to_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum PackageType {
    PackageTypeUnspecified = 0,
    PackageTypeCursorProject = 1,
    PackageTypeCursorPersonal = 2,
    PackageTypeClaudeSkill = 3,
    PackageTypeClaudePlugin = 4,
}

impl Default for PackageType {
    fn default() -> Self {
        Self::PackageTypeUnspecified
    }
}

impl PackageType {
    pub fn from_i32(val: i32) -> Self {
        match val {
            1 => Self::PackageTypeCursorProject,
            2 => Self::PackageTypeCursorPersonal,
            3 => Self::PackageTypeClaudeSkill,
            4 => Self::PackageTypeClaudePlugin,
            _ => Self::PackageTypeUnspecified,
        }
    }

    pub fn to_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum SandboxPolicy_Type {
    TypeUnspecified = 0,
    TypeInsecureNone = 1,
    TypeWorkspaceReadwrite = 2,
    TypeWorkspaceReadonly = 3,
}

impl Default for SandboxPolicy_Type {
    fn default() -> Self {
        Self::TypeUnspecified
    }
}

impl SandboxPolicy_Type {
    pub fn from_i32(val: i32) -> Self {
        match val {
            1 => Self::TypeInsecureNone,
            2 => Self::TypeWorkspaceReadwrite,
            3 => Self::TypeWorkspaceReadonly,
            _ => Self::TypeUnspecified,
        }
    }

    pub fn to_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum TimeoutBehavior {
    TimeoutBehaviorUnspecified = 0,
    TimeoutBehaviorCancel = 1,
    TimeoutBehaviorBackground = 2,
}

impl Default for TimeoutBehavior {
    fn default() -> Self {
        Self::TimeoutBehaviorUnspecified
    }
}

impl TimeoutBehavior {
    pub fn from_i32(val: i32) -> Self {
        match val {
            1 => Self::TimeoutBehaviorCancel,
            2 => Self::TimeoutBehaviorBackground,
            _ => Self::TimeoutBehaviorUnspecified,
        }
    }

    pub fn to_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum ShellAbortReason {
    ShellAbortReasonUnspecified = 0,
    ShellAbortReasonUserAbort = 1,
    ShellAbortReasonTimeout = 2,
}

impl Default for ShellAbortReason {
    fn default() -> Self {
        Self::ShellAbortReasonUnspecified
    }
}

impl ShellAbortReason {
    pub fn from_i32(val: i32) -> Self {
        match val {
            1 => Self::ShellAbortReasonUserAbort,
            2 => Self::ShellAbortReasonTimeout,
            _ => Self::ShellAbortReasonUnspecified,
        }
    }

    pub fn to_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum CustomSubagentPermissionMode {
    CustomSubagentPermissionModeUnspecified = 0,
    CustomSubagentPermissionModeDefault = 1,
    CustomSubagentPermissionModeReadonly = 2,
}

impl Default for CustomSubagentPermissionMode {
    fn default() -> Self {
        Self::CustomSubagentPermissionModeUnspecified
    }
}

impl CustomSubagentPermissionMode {
    pub fn from_i32(val: i32) -> Self {
        match val {
            1 => Self::CustomSubagentPermissionModeDefault,
            2 => Self::CustomSubagentPermissionModeReadonly,
            _ => Self::CustomSubagentPermissionModeUnspecified,
        }
    }

    pub fn to_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum TodoStatus {
    TodoStatusUnspecified = 0,
    TodoStatusPending = 1,
    TodoStatusInProgress = 2,
    TodoStatusCompleted = 3,
    TodoStatusCancelled = 4,
}

impl Default for TodoStatus {
    fn default() -> Self {
        Self::TodoStatusUnspecified
    }
}

impl TodoStatus {
    pub fn from_i32(val: i32) -> Self {
        match val {
            1 => Self::TodoStatusPending,
            2 => Self::TodoStatusInProgress,
            3 => Self::TodoStatusCompleted,
            4 => Self::TodoStatusCancelled,
            _ => Self::TodoStatusUnspecified,
        }
    }

    pub fn to_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum ClientOS {
    ClientOsUnspecified = 0,
    ClientOsWindows = 1,
    ClientOsMacos = 2,
    ClientOsLinux = 3,
}

impl Default for ClientOS {
    fn default() -> Self {
        Self::ClientOsUnspecified
    }
}

impl ClientOS {
    pub fn from_i32(val: i32) -> Self {
        match val {
            1 => Self::ClientOsWindows,
            2 => Self::ClientOsMacos,
            3 => Self::ClientOsLinux,
            _ => Self::ClientOsUnspecified,
        }
    }

    pub fn to_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum ArtifactUploadDispatchStatus {
    ArtifactUploadDispatchStatusUnspecified = 0,
    ArtifactUploadDispatchStatusAccepted = 1,
    ArtifactUploadDispatchStatusRejected = 2,
    ArtifactUploadDispatchStatusSkippedAlreadyInProgress = 3,
}

impl Default for ArtifactUploadDispatchStatus {
    fn default() -> Self {
        Self::ArtifactUploadDispatchStatusUnspecified
    }
}

impl ArtifactUploadDispatchStatus {
    pub fn from_i32(val: i32) -> Self {
        match val {
            1 => Self::ArtifactUploadDispatchStatusAccepted,
            2 => Self::ArtifactUploadDispatchStatusRejected,
            3 => Self::ArtifactUploadDispatchStatusSkippedAlreadyInProgress,
            _ => Self::ArtifactUploadDispatchStatusUnspecified,
        }
    }

    pub fn to_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum Frame_Kind {
    KindUnspecified = 0,
    KindRequest = 1,
    KindResponse = 2,
    KindError = 3,
}

impl Default for Frame_Kind {
    fn default() -> Self {
        Self::KindUnspecified
    }
}

impl Frame_Kind {
    pub fn from_i32(val: i32) -> Self {
        match val {
            1 => Self::KindRequest,
            2 => Self::KindResponse,
            3 => Self::KindError,
            _ => Self::KindUnspecified,
        }
    }

    pub fn to_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum BugbotDeeplinkEventKind {
    BugbotDeeplinkEventKindUnspecified = 0,
    BugbotDeeplinkEventKindClicked = 1,
    BugbotDeeplinkEventKindHandledDialogShown = 2,
    BugbotDeeplinkEventKindHandledChatCreated = 3,
    BugbotDeeplinkEventKindError = 4,
    BugbotDeeplinkEventKindHandledFixInWeb = 5,
}

impl Default for BugbotDeeplinkEventKind {
    fn default() -> Self {
        Self::BugbotDeeplinkEventKindUnspecified
    }
}

impl BugbotDeeplinkEventKind {
    pub fn from_i32(val: i32) -> Self {
        match val {
            1 => Self::BugbotDeeplinkEventKindClicked,
            2 => Self::BugbotDeeplinkEventKindHandledDialogShown,
            3 => Self::BugbotDeeplinkEventKindHandledChatCreated,
            4 => Self::BugbotDeeplinkEventKindError,
            5 => Self::BugbotDeeplinkEventKindHandledFixInWeb,
            _ => Self::BugbotDeeplinkEventKindUnspecified,
        }
    }

    pub fn to_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GlobToolResultResult {
    Success(GlobToolSuccess),
    Error(GlobToolError),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GlobToolResult {
    pub result: Option<GlobToolResultResult>,
}

impl GlobToolResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                GlobToolResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                GlobToolResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = GlobToolSuccess::decode(&field.value)?;
                    msg.result = Some(GlobToolResultResult::Success(val));
                }
                2 => {
                    let val = GlobToolError::decode(&field.value)?;
                    msg.result = Some(GlobToolResultResult::Error(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GlobToolError {
    pub error: String,
}

impl GlobToolError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GlobToolSuccess {
    pub pattern: String,
    pub path: String,
    pub files: Vec<String>,
    pub total_files: i32,
    pub client_truncated: bool,
    pub ripgrep_truncated: bool,
}

impl GlobToolSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.pattern));
        chunks.push(encode_string_field(2, &self.path));
        chunks.push(encode_repeated_string_field(3, &self.files));
        if self.total_files != 0 {
            chunks.push(encode_varint_field_always(4, self.total_files as u64));
        }
        chunks.push(encode_bool_field(5, self.client_truncated));
        chunks.push(encode_bool_field(6, self.ripgrep_truncated));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.pattern = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.files.push(String::from_utf8(field.value).ok()?);
                }
                4 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.total_files = val as i32;
                }
                5 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.client_truncated = val != 0;
                }
                6 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.ripgrep_truncated = val != 0;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GlobToolCall {
    pub args: Vec<u8>,
    pub result: Option<GlobToolResult>,
}

impl GlobToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if !self.args.is_empty() {
            chunks.push(encode_message_field(1, &self.args));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = field.value;
                }
                2 => {
                    msg.result = Some(GlobToolResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadLintsToolCall {
    pub args: Option<ReadLintsToolArgs>,
    pub result: Option<ReadLintsToolResult>,
}

impl ReadLintsToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(ReadLintsToolArgs::decode(&field.value)?);
                }
                2 => {
                    msg.result = Some(ReadLintsToolResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadLintsToolArgs {
    pub paths: Vec<String>,
}

impl ReadLintsToolArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_repeated_string_field(1, &self.paths));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.paths.push(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReadLintsToolResultResult {
    Success(ReadLintsToolSuccess),
    Error(ReadLintsToolError),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadLintsToolResult {
    pub result: Option<ReadLintsToolResultResult>,
}

impl ReadLintsToolResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                ReadLintsToolResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                ReadLintsToolResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = ReadLintsToolSuccess::decode(&field.value)?;
                    msg.result = Some(ReadLintsToolResultResult::Success(val));
                }
                2 => {
                    let val = ReadLintsToolError::decode(&field.value)?;
                    msg.result = Some(ReadLintsToolResultResult::Error(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadLintsToolSuccess {
    pub file_diagnostics: Vec<FileDiagnostics>,
    pub total_files: i32,
    pub total_diagnostics: i32,
}

impl ReadLintsToolSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        let items_file_diagnostics: Vec<Vec<u8>> = self
            .file_diagnostics
            .iter()
            .map(|item| item.encode())
            .collect();
        chunks.push(encode_repeated_message_field(1, &items_file_diagnostics));
        if self.total_files != 0 {
            chunks.push(encode_varint_field_always(2, self.total_files as u64));
        }
        if self.total_diagnostics != 0 {
            chunks.push(encode_varint_field_always(3, self.total_diagnostics as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.file_diagnostics
                        .push(FileDiagnostics::decode(&field.value)?);
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.total_files = val as i32;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.total_diagnostics = val as i32;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FileDiagnostics {
    pub path: String,
    pub diagnostics: Vec<DiagnosticItem>,
    pub diagnostics_count: i32,
}

impl FileDiagnostics {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        let items_diagnostics: Vec<Vec<u8>> =
            self.diagnostics.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(2, &items_diagnostics));
        if self.diagnostics_count != 0 {
            chunks.push(encode_varint_field_always(3, self.diagnostics_count as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.diagnostics.push(DiagnosticItem::decode(&field.value)?);
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.diagnostics_count = val as i32;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DiagnosticItem {
    pub severity: DiagnosticSeverity,
    pub range: Option<DiagnosticRange>,
    pub message: String,
    pub source: String,
    pub code: String,
    pub is_stale: bool,
}

impl DiagnosticItem {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.severity != Default::default() {
            chunks.push(encode_int32_field(1, self.severity.to_i32() as u32));
        }
        if let Some(ref val) = self.range {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        chunks.push(encode_string_field(3, &self.message));
        chunks.push(encode_string_field(4, &self.source));
        chunks.push(encode_string_field(5, &self.code));
        chunks.push(encode_bool_field(6, self.is_stale));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    let enum_val = DiagnosticSeverity::from_i32(val as i32);
                    msg.severity = enum_val;
                }
                2 => {
                    msg.range = Some(DiagnosticRange::decode(&field.value)?);
                }
                3 => {
                    msg.message = String::from_utf8(field.value).ok()?;
                }
                4 => {
                    msg.source = String::from_utf8(field.value).ok()?;
                }
                5 => {
                    msg.code = String::from_utf8(field.value).ok()?;
                }
                6 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.is_stale = val != 0;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DiagnosticRange {
    pub start: Option<Position>,
    pub end: Option<Position>,
}

impl DiagnosticRange {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.start {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.end {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.start = Some(Position::decode(&field.value)?);
                }
                2 => {
                    msg.end = Some(Position::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadLintsToolError {
    pub error_message: String,
}

impl ReadLintsToolError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.error_message));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.error_message = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct McpToolError {
    pub error: String,
}

impl McpToolError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum McpToolResultResult {
    Success(McpSuccess),
    Error(McpToolError),
    Rejected(McpRejected),
    PermissionDenied(McpPermissionDenied),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct McpToolResult {
    pub result: Option<McpToolResultResult>,
}

impl McpToolResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                McpToolResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                McpToolResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                McpToolResultResult::Rejected(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
                McpToolResultResult::PermissionDenied(ref inner) => {
                    chunks.push(encode_message_field(4, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = McpSuccess::decode(&field.value)?;
                    msg.result = Some(McpToolResultResult::Success(val));
                }
                2 => {
                    let val = McpToolError::decode(&field.value)?;
                    msg.result = Some(McpToolResultResult::Error(val));
                }
                3 => {
                    let val = McpRejected::decode(&field.value)?;
                    msg.result = Some(McpToolResultResult::Rejected(val));
                }
                4 => {
                    let val = McpPermissionDenied::decode(&field.value)?;
                    msg.result = Some(McpToolResultResult::PermissionDenied(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct McpToolCall {
    pub args: Option<McpArgs>,
    pub result: Option<McpToolResult>,
}

impl McpToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(McpArgs::decode(&field.value)?);
                }
                2 => {
                    msg.result = Some(McpToolResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SemSearchToolCall {
    pub args: Option<SemSearchToolArgs>,
    pub result: Option<SemSearchToolResult>,
}

impl SemSearchToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(SemSearchToolArgs::decode(&field.value)?);
                }
                2 => {
                    msg.result = Some(SemSearchToolResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SemSearchToolArgs {
    pub query: String,
    pub target_directories: Vec<String>,
    pub explanation: String,
}

impl SemSearchToolArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.query));
        chunks.push(encode_repeated_string_field(2, &self.target_directories));
        chunks.push(encode_string_field(3, &self.explanation));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.query = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.target_directories
                        .push(String::from_utf8(field.value).ok()?);
                }
                3 => {
                    msg.explanation = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SemSearchToolResultResult {
    Success(SemSearchToolSuccess),
    Error(SemSearchToolError),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SemSearchToolResult {
    pub result: Option<SemSearchToolResultResult>,
}

impl SemSearchToolResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                SemSearchToolResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                SemSearchToolResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = SemSearchToolSuccess::decode(&field.value)?;
                    msg.result = Some(SemSearchToolResultResult::Success(val));
                }
                2 => {
                    let val = SemSearchToolError::decode(&field.value)?;
                    msg.result = Some(SemSearchToolResultResult::Error(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SemSearchToolSuccess {
    pub results: String,
    pub code_results: Vec<Vec<u8>>,
}

impl SemSearchToolSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.results));
        chunks.push(encode_repeated_message_field(2, &self.code_results));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.results = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.code_results.push(field.value);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SemSearchToolError {
    pub error_message: String,
}

impl SemSearchToolError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.error_message));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.error_message = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ListMcpResourcesToolCall {
    pub args: Option<ListMcpResourcesExecArgs>,
    pub result: Option<ListMcpResourcesExecResult>,
}

impl ListMcpResourcesToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(ListMcpResourcesExecArgs::decode(&field.value)?);
                }
                2 => {
                    msg.result = Some(ListMcpResourcesExecResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadMcpResourceToolCall {
    pub args: Option<ReadMcpResourceExecArgs>,
    pub result: Option<ReadMcpResourceExecResult>,
}

impl ReadMcpResourceToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(ReadMcpResourceExecArgs::decode(&field.value)?);
                }
                2 => {
                    msg.result = Some(ReadMcpResourceExecResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FetchToolCall {
    pub args: Option<FetchArgs>,
    pub result: Option<FetchResult>,
}

impl FetchToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(FetchArgs::decode(&field.value)?);
                }
                2 => {
                    msg.result = Some(FetchResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RecordScreenToolCall {
    pub args: Option<RecordScreenArgs>,
    pub result: Option<RecordScreenResult>,
}

impl RecordScreenToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(RecordScreenArgs::decode(&field.value)?);
                }
                2 => {
                    msg.result = Some(RecordScreenResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WriteShellStdinToolCall {
    pub args: Option<WriteShellStdinArgs>,
    pub result: Option<WriteShellStdinResult>,
}

impl WriteShellStdinToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(WriteShellStdinArgs::decode(&field.value)?);
                }
                2 => {
                    msg.result = Some(WriteShellStdinResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReflectArgs {
    pub unexpected_action_outcomes: String,
    pub relevant_instructions: String,
    pub scenario_analysis: String,
    pub critical_synthesis: String,
    pub next_steps: String,
    pub tool_call_id: String,
}

impl ReflectArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.unexpected_action_outcomes));
        chunks.push(encode_string_field(2, &self.relevant_instructions));
        chunks.push(encode_string_field(3, &self.scenario_analysis));
        chunks.push(encode_string_field(4, &self.critical_synthesis));
        chunks.push(encode_string_field(5, &self.next_steps));
        chunks.push(encode_string_field(6, &self.tool_call_id));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.unexpected_action_outcomes = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.relevant_instructions = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.scenario_analysis = String::from_utf8(field.value).ok()?;
                }
                4 => {
                    msg.critical_synthesis = String::from_utf8(field.value).ok()?;
                }
                5 => {
                    msg.next_steps = String::from_utf8(field.value).ok()?;
                }
                6 => {
                    msg.tool_call_id = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReflectResultResult {
    Success(ReflectSuccess),
    Error(ReflectError),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReflectResult {
    pub result: Option<ReflectResultResult>,
}

impl ReflectResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                ReflectResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                ReflectResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = ReflectSuccess::decode(&field.value)?;
                    msg.result = Some(ReflectResultResult::Success(val));
                }
                2 => {
                    let val = ReflectError::decode(&field.value)?;
                    msg.result = Some(ReflectResultResult::Error(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReflectSuccess {}

impl ReflectSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReflectError {
    pub error: String,
}

impl ReflectError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReflectToolCall {
    pub args: Option<ReflectArgs>,
    pub result: Option<ReflectResult>,
}

impl ReflectToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(ReflectArgs::decode(&field.value)?);
                }
                2 => {
                    msg.result = Some(ReflectResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct StartGrindExecutionArgs {
    pub explanation: Option<String>,
    pub tool_call_id: String,
}

impl StartGrindExecutionArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.explanation {
            chunks.push(encode_string_field_always(1, val));
        }
        chunks.push(encode_string_field(2, &self.tool_call_id));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.explanation = Some(String::from_utf8(field.value).ok()?);
                }
                2 => {
                    msg.tool_call_id = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StartGrindExecutionResultResult {
    Success(StartGrindExecutionSuccess),
    Error(StartGrindExecutionError),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct StartGrindExecutionResult {
    pub result: Option<StartGrindExecutionResultResult>,
}

impl StartGrindExecutionResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                StartGrindExecutionResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                StartGrindExecutionResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = StartGrindExecutionSuccess::decode(&field.value)?;
                    msg.result = Some(StartGrindExecutionResultResult::Success(val));
                }
                2 => {
                    let val = StartGrindExecutionError::decode(&field.value)?;
                    msg.result = Some(StartGrindExecutionResultResult::Error(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct StartGrindExecutionSuccess {}

impl StartGrindExecutionSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct StartGrindExecutionError {
    pub error: String,
}

impl StartGrindExecutionError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct StartGrindExecutionToolCall {
    pub args: Option<StartGrindExecutionArgs>,
    pub result: Option<StartGrindExecutionResult>,
}

impl StartGrindExecutionToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(StartGrindExecutionArgs::decode(&field.value)?);
                }
                2 => {
                    msg.result = Some(StartGrindExecutionResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct StartGrindPlanningArgs {
    pub explanation: Option<String>,
    pub tool_call_id: String,
}

impl StartGrindPlanningArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.explanation {
            chunks.push(encode_string_field_always(1, val));
        }
        chunks.push(encode_string_field(2, &self.tool_call_id));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.explanation = Some(String::from_utf8(field.value).ok()?);
                }
                2 => {
                    msg.tool_call_id = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StartGrindPlanningResultResult {
    Success(StartGrindPlanningSuccess),
    Error(StartGrindPlanningError),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct StartGrindPlanningResult {
    pub result: Option<StartGrindPlanningResultResult>,
}

impl StartGrindPlanningResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                StartGrindPlanningResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                StartGrindPlanningResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = StartGrindPlanningSuccess::decode(&field.value)?;
                    msg.result = Some(StartGrindPlanningResultResult::Success(val));
                }
                2 => {
                    let val = StartGrindPlanningError::decode(&field.value)?;
                    msg.result = Some(StartGrindPlanningResultResult::Error(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct StartGrindPlanningSuccess {}

impl StartGrindPlanningSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct StartGrindPlanningError {
    pub error: String,
}

impl StartGrindPlanningError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct StartGrindPlanningToolCall {
    pub args: Option<StartGrindPlanningArgs>,
    pub result: Option<StartGrindPlanningResult>,
}

impl StartGrindPlanningToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(StartGrindPlanningArgs::decode(&field.value)?);
                }
                2 => {
                    msg.result = Some(StartGrindPlanningResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TaskArgs {
    pub description: String,
    pub prompt: String,
    pub subagent_type: Option<SubagentType>,
    pub model: Option<String>,
    pub resume: Option<String>,
}

impl TaskArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.description));
        chunks.push(encode_string_field(2, &self.prompt));
        if let Some(ref val) = self.subagent_type {
            chunks.push(encode_message_field(3, &val.encode()));
        }
        if let Some(ref val) = self.model {
            chunks.push(encode_string_field_always(4, val));
        }
        if let Some(ref val) = self.resume {
            chunks.push(encode_string_field_always(5, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.description = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.prompt = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.subagent_type = Some(SubagentType::decode(&field.value)?);
                }
                4 => {
                    msg.model = Some(String::from_utf8(field.value).ok()?);
                }
                5 => {
                    msg.resume = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TaskSuccess {
    pub conversation_steps: Vec<ConversationStep>,
    pub agent_id: Option<String>,
    pub is_background: bool,
    pub duration_ms: Option<u64>,
}

impl TaskSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        let items_conversation_steps: Vec<Vec<u8>> = self
            .conversation_steps
            .iter()
            .map(|item| item.encode())
            .collect();
        chunks.push(encode_repeated_message_field(1, &items_conversation_steps));
        if let Some(ref val) = self.agent_id {
            chunks.push(encode_string_field_always(2, val));
        }
        chunks.push(encode_bool_field(3, self.is_background));
        if let Some(ref val) = self.duration_ms {
            chunks.push(encode_varint_field_always(4, *val as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.conversation_steps
                        .push(ConversationStep::decode(&field.value)?);
                }
                2 => {
                    msg.agent_id = Some(String::from_utf8(field.value).ok()?);
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.is_background = val != 0;
                }
                4 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.duration_ms = Some(val as u64);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TaskError {
    pub error: String,
}

impl TaskError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskResultResult {
    Success(TaskSuccess),
    Error(TaskError),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TaskResult {
    pub result: Option<TaskResultResult>,
}

impl TaskResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                TaskResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                TaskResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = TaskSuccess::decode(&field.value)?;
                    msg.result = Some(TaskResultResult::Success(val));
                }
                2 => {
                    let val = TaskError::decode(&field.value)?;
                    msg.result = Some(TaskResultResult::Error(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TaskToolCall {
    pub args: Option<TaskArgs>,
    pub result: Option<TaskResult>,
}

impl TaskToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(TaskArgs::decode(&field.value)?);
                }
                2 => {
                    msg.result = Some(TaskResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TaskToolCallDelta {
    pub interaction_update: Option<Box<InteractionUpdate>>,
}

impl TaskToolCallDelta {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.interaction_update {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.interaction_update =
                        Some(Box::new(InteractionUpdate::decode(&field.value)?));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToolCallTool {
    ShellToolCall(ShellToolCall),
    DeleteToolCall(DeleteToolCall),
    GlobToolCall(GlobToolCall),
    GrepToolCall(GrepToolCall),
    ReadToolCall(ReadToolCall),
    UpdateTodosToolCall(UpdateTodosToolCall),
    ReadTodosToolCall(ReadTodosToolCall),
    EditToolCall(EditToolCall),
    LsToolCall(LsToolCall),
    ReadLintsToolCall(ReadLintsToolCall),
    McpToolCall(McpToolCall),
    SemSearchToolCall(SemSearchToolCall),
    CreatePlanToolCall(CreatePlanToolCall),
    WebSearchToolCall(WebSearchToolCall),
    TaskToolCall(TaskToolCall),
    ListMcpResourcesToolCall(ListMcpResourcesToolCall),
    ReadMcpResourceToolCall(ReadMcpResourceToolCall),
    ApplyAgentDiffToolCall(ApplyAgentDiffToolCall),
    AskQuestionToolCall(AskQuestionToolCall),
    FetchToolCall(FetchToolCall),
    SwitchModeToolCall(SwitchModeToolCall),
    ExaSearchToolCall(ExaSearchToolCall),
    ExaFetchToolCall(ExaFetchToolCall),
    GenerateImageToolCall(GenerateImageToolCall),
    RecordScreenToolCall(RecordScreenToolCall),
    ComputerUseToolCall(ComputerUseToolCall),
    WriteShellStdinToolCall(WriteShellStdinToolCall),
    ReflectToolCall(ReflectToolCall),
    SetupVmEnvironmentToolCall(SetupVmEnvironmentToolCall),
    TruncatedToolCall(TruncatedToolCall),
    StartGrindExecutionToolCall(StartGrindExecutionToolCall),
    StartGrindPlanningToolCall(StartGrindPlanningToolCall),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ToolCall {
    pub tool: Option<ToolCallTool>,
}

impl ToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.tool {
            match val {
                ToolCallTool::ShellToolCall(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                ToolCallTool::DeleteToolCall(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
                ToolCallTool::GlobToolCall(ref inner) => {
                    chunks.push(encode_message_field(4, &inner.encode()));
                }
                ToolCallTool::GrepToolCall(ref inner) => {
                    chunks.push(encode_message_field(5, &inner.encode()));
                }
                ToolCallTool::ReadToolCall(ref inner) => {
                    chunks.push(encode_message_field(8, &inner.encode()));
                }
                ToolCallTool::UpdateTodosToolCall(ref inner) => {
                    chunks.push(encode_message_field(9, &inner.encode()));
                }
                ToolCallTool::ReadTodosToolCall(ref inner) => {
                    chunks.push(encode_message_field(10, &inner.encode()));
                }
                ToolCallTool::EditToolCall(ref inner) => {
                    chunks.push(encode_message_field(12, &inner.encode()));
                }
                ToolCallTool::LsToolCall(ref inner) => {
                    chunks.push(encode_message_field(13, &inner.encode()));
                }
                ToolCallTool::ReadLintsToolCall(ref inner) => {
                    chunks.push(encode_message_field(14, &inner.encode()));
                }
                ToolCallTool::McpToolCall(ref inner) => {
                    chunks.push(encode_message_field(15, &inner.encode()));
                }
                ToolCallTool::SemSearchToolCall(ref inner) => {
                    chunks.push(encode_message_field(16, &inner.encode()));
                }
                ToolCallTool::CreatePlanToolCall(ref inner) => {
                    chunks.push(encode_message_field(17, &inner.encode()));
                }
                ToolCallTool::WebSearchToolCall(ref inner) => {
                    chunks.push(encode_message_field(18, &inner.encode()));
                }
                ToolCallTool::TaskToolCall(ref inner) => {
                    chunks.push(encode_message_field(19, &inner.encode()));
                }
                ToolCallTool::ListMcpResourcesToolCall(ref inner) => {
                    chunks.push(encode_message_field(20, &inner.encode()));
                }
                ToolCallTool::ReadMcpResourceToolCall(ref inner) => {
                    chunks.push(encode_message_field(21, &inner.encode()));
                }
                ToolCallTool::ApplyAgentDiffToolCall(ref inner) => {
                    chunks.push(encode_message_field(22, &inner.encode()));
                }
                ToolCallTool::AskQuestionToolCall(ref inner) => {
                    chunks.push(encode_message_field(23, &inner.encode()));
                }
                ToolCallTool::FetchToolCall(ref inner) => {
                    chunks.push(encode_message_field(24, &inner.encode()));
                }
                ToolCallTool::SwitchModeToolCall(ref inner) => {
                    chunks.push(encode_message_field(25, &inner.encode()));
                }
                ToolCallTool::ExaSearchToolCall(ref inner) => {
                    chunks.push(encode_message_field(26, &inner.encode()));
                }
                ToolCallTool::ExaFetchToolCall(ref inner) => {
                    chunks.push(encode_message_field(27, &inner.encode()));
                }
                ToolCallTool::GenerateImageToolCall(ref inner) => {
                    chunks.push(encode_message_field(28, &inner.encode()));
                }
                ToolCallTool::RecordScreenToolCall(ref inner) => {
                    chunks.push(encode_message_field(29, &inner.encode()));
                }
                ToolCallTool::ComputerUseToolCall(ref inner) => {
                    chunks.push(encode_message_field(30, &inner.encode()));
                }
                ToolCallTool::WriteShellStdinToolCall(ref inner) => {
                    chunks.push(encode_message_field(31, &inner.encode()));
                }
                ToolCallTool::ReflectToolCall(ref inner) => {
                    chunks.push(encode_message_field(32, &inner.encode()));
                }
                ToolCallTool::SetupVmEnvironmentToolCall(ref inner) => {
                    chunks.push(encode_message_field(33, &inner.encode()));
                }
                ToolCallTool::TruncatedToolCall(ref inner) => {
                    chunks.push(encode_message_field(34, &inner.encode()));
                }
                ToolCallTool::StartGrindExecutionToolCall(ref inner) => {
                    chunks.push(encode_message_field(35, &inner.encode()));
                }
                ToolCallTool::StartGrindPlanningToolCall(ref inner) => {
                    chunks.push(encode_message_field(36, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = ShellToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::ShellToolCall(val));
                }
                3 => {
                    let val = DeleteToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::DeleteToolCall(val));
                }
                4 => {
                    let val = GlobToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::GlobToolCall(val));
                }
                5 => {
                    let val = GrepToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::GrepToolCall(val));
                }
                8 => {
                    let val = ReadToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::ReadToolCall(val));
                }
                9 => {
                    let val = UpdateTodosToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::UpdateTodosToolCall(val));
                }
                10 => {
                    let val = ReadTodosToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::ReadTodosToolCall(val));
                }
                12 => {
                    let val = EditToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::EditToolCall(val));
                }
                13 => {
                    let val = LsToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::LsToolCall(val));
                }
                14 => {
                    let val = ReadLintsToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::ReadLintsToolCall(val));
                }
                15 => {
                    let val = McpToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::McpToolCall(val));
                }
                16 => {
                    let val = SemSearchToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::SemSearchToolCall(val));
                }
                17 => {
                    let val = CreatePlanToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::CreatePlanToolCall(val));
                }
                18 => {
                    let val = WebSearchToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::WebSearchToolCall(val));
                }
                19 => {
                    let val = TaskToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::TaskToolCall(val));
                }
                20 => {
                    let val = ListMcpResourcesToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::ListMcpResourcesToolCall(val));
                }
                21 => {
                    let val = ReadMcpResourceToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::ReadMcpResourceToolCall(val));
                }
                22 => {
                    let val = ApplyAgentDiffToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::ApplyAgentDiffToolCall(val));
                }
                23 => {
                    let val = AskQuestionToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::AskQuestionToolCall(val));
                }
                24 => {
                    let val = FetchToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::FetchToolCall(val));
                }
                25 => {
                    let val = SwitchModeToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::SwitchModeToolCall(val));
                }
                26 => {
                    let val = ExaSearchToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::ExaSearchToolCall(val));
                }
                27 => {
                    let val = ExaFetchToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::ExaFetchToolCall(val));
                }
                28 => {
                    let val = GenerateImageToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::GenerateImageToolCall(val));
                }
                29 => {
                    let val = RecordScreenToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::RecordScreenToolCall(val));
                }
                30 => {
                    let val = ComputerUseToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::ComputerUseToolCall(val));
                }
                31 => {
                    let val = WriteShellStdinToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::WriteShellStdinToolCall(val));
                }
                32 => {
                    let val = ReflectToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::ReflectToolCall(val));
                }
                33 => {
                    let val = SetupVmEnvironmentToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::SetupVmEnvironmentToolCall(val));
                }
                34 => {
                    let val = TruncatedToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::TruncatedToolCall(val));
                }
                35 => {
                    let val = StartGrindExecutionToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::StartGrindExecutionToolCall(val));
                }
                36 => {
                    let val = StartGrindPlanningToolCall::decode(&field.value)?;
                    msg.tool = Some(ToolCallTool::StartGrindPlanningToolCall(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TruncatedToolCallArgs {}

impl TruncatedToolCallArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TruncatedToolCallSuccess {}

impl TruncatedToolCallSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TruncatedToolCallError {
    pub error: String,
}

impl TruncatedToolCallError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TruncatedToolCallResultResult {
    Success(TruncatedToolCallSuccess),
    Error(TruncatedToolCallError),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TruncatedToolCallResult {
    pub result: Option<TruncatedToolCallResultResult>,
}

impl TruncatedToolCallResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                TruncatedToolCallResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                TruncatedToolCallResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = TruncatedToolCallSuccess::decode(&field.value)?;
                    msg.result = Some(TruncatedToolCallResultResult::Success(val));
                }
                2 => {
                    let val = TruncatedToolCallError::decode(&field.value)?;
                    msg.result = Some(TruncatedToolCallResultResult::Error(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TruncatedToolCall {
    pub original_step_blob_id: Vec<u8>,
    pub args: Option<TruncatedToolCallArgs>,
    pub result: Option<TruncatedToolCallResult>,
}

impl TruncatedToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if !self.original_step_blob_id.is_empty() {
            chunks.push(encode_message_field(1, &self.original_step_blob_id));
        }
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(3, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.original_step_blob_id = field.value;
                }
                2 => {
                    msg.args = Some(TruncatedToolCallArgs::decode(&field.value)?);
                }
                3 => {
                    msg.result = Some(TruncatedToolCallResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToolCallDeltaDelta {
    ShellToolCallDelta(ShellToolCallDelta),
    TaskToolCallDelta(Box<TaskToolCallDelta>),
    EditToolCallDelta(EditToolCallDelta),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ToolCallDelta {
    pub delta: Option<ToolCallDeltaDelta>,
}

impl ToolCallDelta {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.delta {
            match val {
                ToolCallDeltaDelta::ShellToolCallDelta(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                ToolCallDeltaDelta::TaskToolCallDelta(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                ToolCallDeltaDelta::EditToolCallDelta(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = ShellToolCallDelta::decode(&field.value)?;
                    msg.delta = Some(ToolCallDeltaDelta::ShellToolCallDelta(val));
                }
                2 => {
                    let val = Box::new(TaskToolCallDelta::decode(&field.value)?);
                    msg.delta = Some(ToolCallDeltaDelta::TaskToolCallDelta(val));
                }
                3 => {
                    let val = EditToolCallDelta::decode(&field.value)?;
                    msg.delta = Some(ToolCallDeltaDelta::EditToolCallDelta(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConversationStepMessage {
    AssistantMessage(AssistantMessage),
    ToolCall(ToolCall),
    ThinkingMessage(ThinkingMessage),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ConversationStep {
    pub message: Option<ConversationStepMessage>,
}

impl ConversationStep {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.message {
            match val {
                ConversationStepMessage::AssistantMessage(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                ConversationStepMessage::ToolCall(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                ConversationStepMessage::ThinkingMessage(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = AssistantMessage::decode(&field.value)?;
                    msg.message = Some(ConversationStepMessage::AssistantMessage(val));
                }
                2 => {
                    let val = ToolCall::decode(&field.value)?;
                    msg.message = Some(ConversationStepMessage::ToolCall(val));
                }
                3 => {
                    let val = ThinkingMessage::decode(&field.value)?;
                    msg.message = Some(ConversationStepMessage::ThinkingMessage(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConversationActionAction {
    UserMessageAction(UserMessageAction),
    ResumeAction(ResumeAction),
    CancelAction(CancelAction),
    SummarizeAction(SummarizeAction),
    ShellCommandAction(ShellCommandAction),
    StartPlanAction(StartPlanAction),
    ExecutePlanAction(ExecutePlanAction),
    AsyncAskQuestionCompletionAction(AsyncAskQuestionCompletionAction),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ConversationAction {
    pub action: Option<ConversationActionAction>,
}

impl ConversationAction {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.action {
            match val {
                ConversationActionAction::UserMessageAction(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                ConversationActionAction::ResumeAction(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                ConversationActionAction::CancelAction(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
                ConversationActionAction::SummarizeAction(ref inner) => {
                    chunks.push(encode_message_field(4, &inner.encode()));
                }
                ConversationActionAction::ShellCommandAction(ref inner) => {
                    chunks.push(encode_message_field(5, &inner.encode()));
                }
                ConversationActionAction::StartPlanAction(ref inner) => {
                    chunks.push(encode_message_field(6, &inner.encode()));
                }
                ConversationActionAction::ExecutePlanAction(ref inner) => {
                    chunks.push(encode_message_field(7, &inner.encode()));
                }
                ConversationActionAction::AsyncAskQuestionCompletionAction(ref inner) => {
                    chunks.push(encode_message_field(8, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = UserMessageAction::decode(&field.value)?;
                    msg.action = Some(ConversationActionAction::UserMessageAction(val));
                }
                2 => {
                    let val = ResumeAction::decode(&field.value)?;
                    msg.action = Some(ConversationActionAction::ResumeAction(val));
                }
                3 => {
                    let val = CancelAction::decode(&field.value)?;
                    msg.action = Some(ConversationActionAction::CancelAction(val));
                }
                4 => {
                    let val = SummarizeAction::decode(&field.value)?;
                    msg.action = Some(ConversationActionAction::SummarizeAction(val));
                }
                5 => {
                    let val = ShellCommandAction::decode(&field.value)?;
                    msg.action = Some(ConversationActionAction::ShellCommandAction(val));
                }
                6 => {
                    let val = StartPlanAction::decode(&field.value)?;
                    msg.action = Some(ConversationActionAction::StartPlanAction(val));
                }
                7 => {
                    let val = ExecutePlanAction::decode(&field.value)?;
                    msg.action = Some(ConversationActionAction::ExecutePlanAction(val));
                }
                8 => {
                    let val = AsyncAskQuestionCompletionAction::decode(&field.value)?;
                    msg.action = Some(ConversationActionAction::AsyncAskQuestionCompletionAction(
                        val,
                    ));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct UserMessageAction {
    pub user_message: Option<UserMessage>,
    pub request_context: Option<RequestContext>,
    pub send_to_interaction_listener: Option<bool>,
}

impl UserMessageAction {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.user_message {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.request_context {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        if let Some(ref val) = self.send_to_interaction_listener {
            chunks.push(encode_bool_field_always(3, *val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.user_message = Some(UserMessage::decode(&field.value)?);
                }
                2 => {
                    msg.request_context = Some(RequestContext::decode(&field.value)?);
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.send_to_interaction_listener = Some(val != 0);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CancelAction {}

impl CancelAction {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ResumeAction {
    pub request_context: Option<RequestContext>,
}

impl ResumeAction {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.request_context {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                2 => {
                    msg.request_context = Some(RequestContext::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AsyncAskQuestionCompletionAction {
    pub original_tool_call_id: String,
    pub original_args: Option<AskQuestionArgs>,
    pub result: Option<AskQuestionResult>,
}

impl AsyncAskQuestionCompletionAction {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.original_tool_call_id));
        if let Some(ref val) = self.original_args {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(3, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.original_tool_call_id = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.original_args = Some(AskQuestionArgs::decode(&field.value)?);
                }
                3 => {
                    msg.result = Some(AskQuestionResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SummarizeAction {}

impl SummarizeAction {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShellCommandAction {
    pub shell_command: Option<ShellCommand>,
    pub exec_id: String,
}

impl ShellCommandAction {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.shell_command {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        chunks.push(encode_string_field(2, &self.exec_id));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.shell_command = Some(ShellCommand::decode(&field.value)?);
                }
                2 => {
                    msg.exec_id = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct StartPlanAction {
    pub user_message: Option<UserMessage>,
    pub request_context: Option<RequestContext>,
    pub is_spec: bool,
}

impl StartPlanAction {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.user_message {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.request_context {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        chunks.push(encode_bool_field(3, self.is_spec));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.user_message = Some(UserMessage::decode(&field.value)?);
                }
                2 => {
                    msg.request_context = Some(RequestContext::decode(&field.value)?);
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.is_spec = val != 0;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExecutePlanAction {
    pub request_context: Option<RequestContext>,
    pub plan: Option<ConversationPlan>,
    pub plan_file_uri: Option<String>,
    pub plan_file_content: Option<String>,
}

impl ExecutePlanAction {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.request_context {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.plan {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        if let Some(ref val) = self.plan_file_uri {
            chunks.push(encode_string_field_always(3, val));
        }
        if let Some(ref val) = self.plan_file_content {
            chunks.push(encode_string_field_always(4, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.request_context = Some(RequestContext::decode(&field.value)?);
                }
                2 => {
                    msg.plan = Some(ConversationPlan::decode(&field.value)?);
                }
                3 => {
                    msg.plan_file_uri = Some(String::from_utf8(field.value).ok()?);
                }
                4 => {
                    msg.plan_file_content = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct UserMessage {
    pub text: String,
    pub message_id: String,
    pub selected_context: Option<SelectedContext>,
    pub mode: i32,
    pub is_simulated_msg: Option<bool>,
    pub best_of_n_group_id: Option<String>,
    pub try_use_best_of_n_promotion: Option<bool>,
    pub rich_text: Option<String>,
}

impl UserMessage {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.text));
        chunks.push(encode_string_field(2, &self.message_id));
        if let Some(ref val) = self.selected_context {
            chunks.push(encode_message_field(3, &val.encode()));
        }
        if self.mode != 0 {
            chunks.push(encode_varint_field_always(4, self.mode as u64));
        }
        if let Some(ref val) = self.is_simulated_msg {
            chunks.push(encode_bool_field_always(5, *val));
        }
        if let Some(ref val) = self.best_of_n_group_id {
            chunks.push(encode_string_field_always(6, val));
        }
        if let Some(ref val) = self.try_use_best_of_n_promotion {
            chunks.push(encode_bool_field_always(7, *val));
        }
        if let Some(ref val) = self.rich_text {
            chunks.push(encode_string_field_always(8, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.text = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.message_id = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.selected_context = Some(SelectedContext::decode(&field.value)?);
                }
                4 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.mode = val as i32;
                }
                5 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.is_simulated_msg = Some(val != 0);
                }
                6 => {
                    msg.best_of_n_group_id = Some(String::from_utf8(field.value).ok()?);
                }
                7 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.try_use_best_of_n_promotion = Some(val != 0);
                }
                8 => {
                    msg.rich_text = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AssistantMessage {
    pub text: String,
}

impl AssistantMessage {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.text));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.text = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ThinkingMessage {
    pub text: String,
    pub duration_ms: u32,
}

impl ThinkingMessage {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.text));
        if self.duration_ms != 0 {
            chunks.push(encode_varint_field_always(2, self.duration_ms as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.text = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.duration_ms = val as u32;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShellCommand {
    pub command: String,
}

impl ShellCommand {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.command));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.command = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShellOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

impl ShellOutput {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.stdout));
        chunks.push(encode_string_field(2, &self.stderr));
        if self.exit_code != 0 {
            chunks.push(encode_varint_field_always(3, self.exit_code as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.stdout = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.stderr = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.exit_code = val as i32;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConversationTurnTurn {
    AgentConversationTurn(AgentConversationTurn),
    ShellConversationTurn(ShellConversationTurn),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ConversationTurn {
    pub turn: Option<ConversationTurnTurn>,
}

impl ConversationTurn {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.turn {
            match val {
                ConversationTurnTurn::AgentConversationTurn(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                ConversationTurnTurn::ShellConversationTurn(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = AgentConversationTurn::decode(&field.value)?;
                    msg.turn = Some(ConversationTurnTurn::AgentConversationTurn(val));
                }
                2 => {
                    let val = ShellConversationTurn::decode(&field.value)?;
                    msg.turn = Some(ConversationTurnTurn::ShellConversationTurn(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ConversationPlan {
    pub plan: String,
}

impl ConversationPlan {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.plan));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.plan = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConversationTurnStructureTurn {
    AgentConversationTurn(AgentConversationTurnStructure),
    ShellConversationTurn(ShellConversationTurnStructure),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ConversationTurnStructure {
    pub turn: Option<ConversationTurnStructureTurn>,
}

impl ConversationTurnStructure {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.turn {
            match val {
                ConversationTurnStructureTurn::AgentConversationTurn(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                ConversationTurnStructureTurn::ShellConversationTurn(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = AgentConversationTurnStructure::decode(&field.value)?;
                    msg.turn = Some(ConversationTurnStructureTurn::AgentConversationTurn(val));
                }
                2 => {
                    let val = ShellConversationTurnStructure::decode(&field.value)?;
                    msg.turn = Some(ConversationTurnStructureTurn::ShellConversationTurn(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AgentConversationTurn {
    pub user_message: Option<UserMessage>,
    pub steps: Vec<ConversationStep>,
    pub request_id: Option<String>,
}

impl AgentConversationTurn {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.user_message {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        let items_steps: Vec<Vec<u8>> = self.steps.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(2, &items_steps));
        if let Some(ref val) = self.request_id {
            chunks.push(encode_string_field_always(3, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.user_message = Some(UserMessage::decode(&field.value)?);
                }
                2 => {
                    msg.steps.push(ConversationStep::decode(&field.value)?);
                }
                3 => {
                    msg.request_id = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AgentConversationTurnStructure {
    pub user_message: Vec<u8>,
    pub steps: Vec<Vec<u8>>,
    pub request_id: Option<String>,
}

impl AgentConversationTurnStructure {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if !self.user_message.is_empty() {
            chunks.push(encode_message_field(1, &self.user_message));
        }
        chunks.push(encode_repeated_message_field(2, &self.steps));
        if let Some(ref val) = self.request_id {
            chunks.push(encode_string_field_always(3, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.user_message = field.value;
                }
                2 => {
                    msg.steps.push(field.value);
                }
                3 => {
                    msg.request_id = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShellConversationTurn {
    pub shell_command: Option<ShellCommand>,
    pub shell_output: Option<ShellOutput>,
}

impl ShellConversationTurn {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.shell_command {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.shell_output {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.shell_command = Some(ShellCommand::decode(&field.value)?);
                }
                2 => {
                    msg.shell_output = Some(ShellOutput::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShellConversationTurnStructure {
    pub shell_command: Vec<u8>,
    pub shell_output: Vec<u8>,
}

impl ShellConversationTurnStructure {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if !self.shell_command.is_empty() {
            chunks.push(encode_message_field(1, &self.shell_command));
        }
        if !self.shell_output.is_empty() {
            chunks.push(encode_message_field(2, &self.shell_output));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.shell_command = field.value;
                }
                2 => {
                    msg.shell_output = field.value;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ConversationSummary {
    pub summary: String,
}

impl ConversationSummary {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.summary));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.summary = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ConversationSummaryArchive {
    pub summarized_messages: Vec<Vec<u8>>,
    pub summary: String,
    pub window_tail: u32,
    pub summary_message: Vec<u8>,
}

impl ConversationSummaryArchive {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_repeated_message_field(1, &self.summarized_messages));
        chunks.push(encode_string_field(2, &self.summary));
        if self.window_tail != 0 {
            chunks.push(encode_varint_field_always(3, self.window_tail as u64));
        }
        if !self.summary_message.is_empty() {
            chunks.push(encode_message_field(4, &self.summary_message));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.summarized_messages.push(field.value);
                }
                2 => {
                    msg.summary = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.window_tail = val as u32;
                }
                4 => {
                    msg.summary_message = field.value;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ConversationTokenDetails {
    pub used_tokens: u32,
    pub max_tokens: u32,
}

impl ConversationTokenDetails {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.used_tokens != 0 {
            chunks.push(encode_varint_field_always(1, self.used_tokens as u64));
        }
        if self.max_tokens != 0 {
            chunks.push(encode_varint_field_always(2, self.max_tokens as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.used_tokens = val as u32;
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.max_tokens = val as u32;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FileState {
    pub content: Option<String>,
    pub initial_content: Option<String>,
}

impl FileState {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.content {
            chunks.push(encode_string_field_always(1, val));
        }
        if let Some(ref val) = self.initial_content {
            chunks.push(encode_string_field_always(2, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.content = Some(String::from_utf8(field.value).ok()?);
                }
                2 => {
                    msg.initial_content = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FileStateStructure {
    pub content: Option<Vec<u8>>,
    pub initial_content: Option<Vec<u8>>,
}

impl FileStateStructure {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.content {
            chunks.push(encode_message_field(1, val));
        }
        if let Some(ref val) = self.initial_content {
            chunks.push(encode_message_field(2, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.content = Some(field.value);
                }
                2 => {
                    msg.initial_content = Some(field.value);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct StepTiming {
    pub duration_ms: u64,
    pub timestamp_ms: u64,
}

impl StepTiming {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.duration_ms != 0 {
            chunks.push(encode_varint_field_always(1, self.duration_ms as u64));
        }
        if self.timestamp_ms != 0 {
            chunks.push(encode_varint_field_always(2, self.timestamp_ms as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.duration_ms = val as u64;
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.timestamp_ms = val as u64;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ConversationState {
    pub root_prompt_messages_json: Vec<String>,
    pub turns: Vec<ConversationTurn>,
    pub todos: Vec<TodoItem>,
    pub pending_tool_calls: Vec<String>,
    pub token_details: Option<ConversationTokenDetails>,
    pub summary: Option<ConversationSummary>,
    pub plan: Option<ConversationPlan>,
    pub summary_archive: Option<ConversationSummaryArchive>,
    pub file_states: std::collections::HashMap<String, FileState>,
    pub summary_archives: Vec<ConversationSummaryArchive>,
}

impl ConversationState {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_repeated_string_field(
            1,
            &self.root_prompt_messages_json,
        ));
        let items_turns: Vec<Vec<u8>> = self.turns.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(8, &items_turns));
        let items_todos: Vec<Vec<u8>> = self.todos.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(3, &items_todos));
        chunks.push(encode_repeated_string_field(4, &self.pending_tool_calls));
        if let Some(ref val) = self.token_details {
            chunks.push(encode_message_field(5, &val.encode()));
        }
        if let Some(ref val) = self.summary {
            chunks.push(encode_message_field(6, &val.encode()));
        }
        if let Some(ref val) = self.plan {
            chunks.push(encode_message_field(7, &val.encode()));
        }
        if let Some(ref val) = self.summary_archive {
            chunks.push(encode_message_field(9, &val.encode()));
        }
        for (key, val) in &self.file_states {
            let mut entry_chunks = Vec::new();
            entry_chunks.push(encode_string_field(1, key));
            entry_chunks.push(encode_message_field(2, &val.encode()));
            chunks.push(encode_message_field(10, &concat_bytes(&entry_chunks)));
        }
        let items_summary_archives: Vec<Vec<u8>> = self
            .summary_archives
            .iter()
            .map(|item| item.encode())
            .collect();
        chunks.push(encode_repeated_message_field(11, &items_summary_archives));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.root_prompt_messages_json
                        .push(String::from_utf8(field.value).ok()?);
                }
                8 => {
                    msg.turns.push(ConversationTurn::decode(&field.value)?);
                }
                3 => {
                    msg.todos.push(TodoItem::decode(&field.value)?);
                }
                4 => {
                    msg.pending_tool_calls
                        .push(String::from_utf8(field.value).ok()?);
                }
                5 => {
                    msg.token_details = Some(ConversationTokenDetails::decode(&field.value)?);
                }
                6 => {
                    msg.summary = Some(ConversationSummary::decode(&field.value)?);
                }
                7 => {
                    msg.plan = Some(ConversationPlan::decode(&field.value)?);
                }
                9 => {
                    msg.summary_archive = Some(ConversationSummaryArchive::decode(&field.value)?);
                }
                10 => {
                    let mut entry_key = String::new();
                    let mut entry_value = <FileState>::default();
                    for sub in parse_proto_fields(&field.value) {
                        match sub.number {
                            1 => entry_key = String::from_utf8(sub.value).ok()?,
                            2 => {
                                entry_value = FileState::decode(&sub.value)?;
                            }
                            _ => {}
                        }
                    }
                    msg.file_states.insert(entry_key, entry_value);
                }
                11 => {
                    msg.summary_archives
                        .push(ConversationSummaryArchive::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SubagentPersistedState {
    pub conversation_state: Option<ConversationStateStructure>,
    pub created_timestamp_ms: u64,
    pub last_used_timestamp_ms: u64,
    pub subagent_type: Option<SubagentType>,
}

impl SubagentPersistedState {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.conversation_state {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if self.created_timestamp_ms != 0 {
            chunks.push(encode_varint_field_always(
                2,
                self.created_timestamp_ms as u64,
            ));
        }
        if self.last_used_timestamp_ms != 0 {
            chunks.push(encode_varint_field_always(
                3,
                self.last_used_timestamp_ms as u64,
            ));
        }
        if let Some(ref val) = self.subagent_type {
            chunks.push(encode_message_field(4, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.conversation_state =
                        Some(ConversationStateStructure::decode(&field.value)?);
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.created_timestamp_ms = val as u64;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.last_used_timestamp_ms = val as u64;
                }
                4 => {
                    msg.subagent_type = Some(SubagentType::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ConversationStateStructure {
    pub turns_old: Vec<Vec<u8>>,
    pub root_prompt_messages_json: Vec<Vec<u8>>,
    pub turns: Vec<Vec<u8>>,
    pub todos: Vec<Vec<u8>>,
    pub pending_tool_calls: Vec<String>,
    pub token_details: Option<ConversationTokenDetails>,
    pub summary: Option<Vec<u8>>,
    pub plan: Option<Vec<u8>>,
    pub previous_workspace_uris: Vec<String>,
    pub mode: Option<i32>,
    pub summary_archive: Option<Vec<u8>>,
    pub file_states: std::collections::HashMap<String, Vec<u8>>,
    pub file_states_v2: std::collections::HashMap<String, FileStateStructure>,
    pub summary_archives: Vec<Vec<u8>>,
    pub turn_timings: Vec<StepTiming>,
    pub subagent_states: std::collections::HashMap<String, SubagentPersistedState>,
    pub self_summary_count: u32,
    pub read_paths: Vec<String>,
}

impl ConversationStateStructure {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_repeated_message_field(2, &self.turns_old));
        chunks.push(encode_repeated_message_field(
            1,
            &self.root_prompt_messages_json,
        ));
        chunks.push(encode_repeated_message_field(8, &self.turns));
        chunks.push(encode_repeated_message_field(3, &self.todos));
        chunks.push(encode_repeated_string_field(4, &self.pending_tool_calls));
        if let Some(ref val) = self.token_details {
            chunks.push(encode_message_field(5, &val.encode()));
        }
        if let Some(ref val) = self.summary {
            chunks.push(encode_message_field(6, val));
        }
        if let Some(ref val) = self.plan {
            chunks.push(encode_message_field(7, val));
        }
        chunks.push(encode_repeated_string_field(
            9,
            &self.previous_workspace_uris,
        ));
        if let Some(ref val) = self.mode {
            chunks.push(encode_varint_field_always(10, *val as u64));
        }
        if let Some(ref val) = self.summary_archive {
            chunks.push(encode_message_field(11, val));
        }
        for (key, val) in &self.file_states {
            let mut entry_chunks = Vec::new();
            entry_chunks.push(encode_string_field(1, key));
            entry_chunks.push(encode_message_field(2, val));
            chunks.push(encode_message_field(12, &concat_bytes(&entry_chunks)));
        }
        for (key, val) in &self.file_states_v2 {
            let mut entry_chunks = Vec::new();
            entry_chunks.push(encode_string_field(1, key));
            entry_chunks.push(encode_message_field(2, &val.encode()));
            chunks.push(encode_message_field(15, &concat_bytes(&entry_chunks)));
        }
        chunks.push(encode_repeated_message_field(13, &self.summary_archives));
        let items_turn_timings: Vec<Vec<u8>> =
            self.turn_timings.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(14, &items_turn_timings));
        for (key, val) in &self.subagent_states {
            let mut entry_chunks = Vec::new();
            entry_chunks.push(encode_string_field(1, key));
            entry_chunks.push(encode_message_field(2, &val.encode()));
            chunks.push(encode_message_field(16, &concat_bytes(&entry_chunks)));
        }
        if self.self_summary_count != 0 {
            chunks.push(encode_varint_field_always(
                17,
                self.self_summary_count as u64,
            ));
        }
        chunks.push(encode_repeated_string_field(18, &self.read_paths));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                2 => {
                    msg.turns_old.push(field.value);
                }
                1 => {
                    msg.root_prompt_messages_json.push(field.value);
                }
                8 => {
                    msg.turns.push(field.value);
                }
                3 => {
                    msg.todos.push(field.value);
                }
                4 => {
                    msg.pending_tool_calls
                        .push(String::from_utf8(field.value).ok()?);
                }
                5 => {
                    msg.token_details = Some(ConversationTokenDetails::decode(&field.value)?);
                }
                6 => {
                    msg.summary = Some(field.value);
                }
                7 => {
                    msg.plan = Some(field.value);
                }
                9 => {
                    msg.previous_workspace_uris
                        .push(String::from_utf8(field.value).ok()?);
                }
                10 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.mode = Some(val as i32);
                }
                11 => {
                    msg.summary_archive = Some(field.value);
                }
                12 => {
                    let mut entry_key = String::new();
                    let mut entry_value = <Vec<u8>>::default();
                    for sub in parse_proto_fields(&field.value) {
                        match sub.number {
                            1 => entry_key = String::from_utf8(sub.value).ok()?,
                            2 => {
                                entry_value = sub.value;
                            }
                            _ => {}
                        }
                    }
                    msg.file_states.insert(entry_key, entry_value);
                }
                15 => {
                    let mut entry_key = String::new();
                    let mut entry_value = <FileStateStructure>::default();
                    for sub in parse_proto_fields(&field.value) {
                        match sub.number {
                            1 => entry_key = String::from_utf8(sub.value).ok()?,
                            2 => {
                                entry_value = FileStateStructure::decode(&sub.value)?;
                            }
                            _ => {}
                        }
                    }
                    msg.file_states_v2.insert(entry_key, entry_value);
                }
                13 => {
                    msg.summary_archives.push(field.value);
                }
                14 => {
                    msg.turn_timings.push(StepTiming::decode(&field.value)?);
                }
                16 => {
                    let mut entry_key = String::new();
                    let mut entry_value = <SubagentPersistedState>::default();
                    for sub in parse_proto_fields(&field.value) {
                        match sub.number {
                            1 => entry_key = String::from_utf8(sub.value).ok()?,
                            2 => {
                                entry_value = SubagentPersistedState::decode(&sub.value)?;
                            }
                            _ => {}
                        }
                    }
                    msg.subagent_states.insert(entry_key, entry_value);
                }
                17 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.self_summary_count = val as u32;
                }
                18 => {
                    msg.read_paths.push(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ThinkingDetails {}

impl ThinkingDetails {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ApiKeyCredentials {
    pub api_key: String,
    pub base_url: Option<String>,
}

impl ApiKeyCredentials {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.api_key));
        if let Some(ref val) = self.base_url {
            chunks.push(encode_string_field_always(2, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.api_key = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.base_url = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AzureCredentials {
    pub api_key: String,
    pub base_url: String,
    pub deployment: String,
}

impl AzureCredentials {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.api_key));
        chunks.push(encode_string_field(2, &self.base_url));
        chunks.push(encode_string_field(3, &self.deployment));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.api_key = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.base_url = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.deployment = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BedrockCredentials {
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
    pub session_token: Option<String>,
}

impl BedrockCredentials {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.access_key));
        chunks.push(encode_string_field(2, &self.secret_key));
        chunks.push(encode_string_field(3, &self.region));
        if let Some(ref val) = self.session_token {
            chunks.push(encode_string_field_always(4, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.access_key = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.secret_key = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.region = String::from_utf8(field.value).ok()?;
                }
                4 => {
                    msg.session_token = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModelDetailsCredentials {
    ApiKeyCredentials(ApiKeyCredentials),
    AzureCredentials(AzureCredentials),
    BedrockCredentials(BedrockCredentials),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ModelDetails {
    pub model_id: String,
    pub display_model_id: String,
    pub display_name: String,
    pub display_name_short: String,
    pub aliases: Vec<String>,
    pub thinking_details: Option<ThinkingDetails>,
    pub max_mode: Option<bool>,
    pub credentials: Option<ModelDetailsCredentials>,
}

impl ModelDetails {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.model_id));
        chunks.push(encode_string_field(3, &self.display_model_id));
        chunks.push(encode_string_field(4, &self.display_name));
        chunks.push(encode_string_field(5, &self.display_name_short));
        chunks.push(encode_repeated_string_field(6, &self.aliases));
        if let Some(ref val) = self.thinking_details {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        if let Some(ref val) = self.max_mode {
            chunks.push(encode_bool_field_always(7, *val));
        }
        if let Some(ref val) = self.credentials {
            match val {
                ModelDetailsCredentials::ApiKeyCredentials(ref inner) => {
                    chunks.push(encode_message_field(8, &inner.encode()));
                }
                ModelDetailsCredentials::AzureCredentials(ref inner) => {
                    chunks.push(encode_message_field(9, &inner.encode()));
                }
                ModelDetailsCredentials::BedrockCredentials(ref inner) => {
                    chunks.push(encode_message_field(10, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.model_id = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.display_model_id = String::from_utf8(field.value).ok()?;
                }
                4 => {
                    msg.display_name = String::from_utf8(field.value).ok()?;
                }
                5 => {
                    msg.display_name_short = String::from_utf8(field.value).ok()?;
                }
                6 => {
                    msg.aliases.push(String::from_utf8(field.value).ok()?);
                }
                2 => {
                    msg.thinking_details = Some(ThinkingDetails::decode(&field.value)?);
                }
                7 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.max_mode = Some(val != 0);
                }
                8 => {
                    let val = ApiKeyCredentials::decode(&field.value)?;
                    msg.credentials = Some(ModelDetailsCredentials::ApiKeyCredentials(val));
                }
                9 => {
                    let val = AzureCredentials::decode(&field.value)?;
                    msg.credentials = Some(ModelDetailsCredentials::AzureCredentials(val));
                }
                10 => {
                    let val = BedrockCredentials::decode(&field.value)?;
                    msg.credentials = Some(ModelDetailsCredentials::BedrockCredentials(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RequestedModelCredentials {
    ApiKeyCredentials(ApiKeyCredentials),
    AzureCredentials(AzureCredentials),
    BedrockCredentials(BedrockCredentials),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RequestedModel {
    pub model_id: String,
    pub max_mode: bool,
    pub parameters: Vec<RequestedModel_ModelParameterbytes>,
    pub credentials: Option<RequestedModelCredentials>,
}

impl RequestedModel {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.model_id));
        chunks.push(encode_bool_field(2, self.max_mode));
        let items_parameters: Vec<Vec<u8>> =
            self.parameters.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(3, &items_parameters));
        if let Some(ref val) = self.credentials {
            match val {
                RequestedModelCredentials::ApiKeyCredentials(ref inner) => {
                    chunks.push(encode_message_field(4, &inner.encode()));
                }
                RequestedModelCredentials::AzureCredentials(ref inner) => {
                    chunks.push(encode_message_field(5, &inner.encode()));
                }
                RequestedModelCredentials::BedrockCredentials(ref inner) => {
                    chunks.push(encode_message_field(6, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.model_id = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.max_mode = val != 0;
                }
                3 => {
                    msg.parameters
                        .push(RequestedModel_ModelParameterbytes::decode(&field.value)?);
                }
                4 => {
                    let val = ApiKeyCredentials::decode(&field.value)?;
                    msg.credentials = Some(RequestedModelCredentials::ApiKeyCredentials(val));
                }
                5 => {
                    let val = AzureCredentials::decode(&field.value)?;
                    msg.credentials = Some(RequestedModelCredentials::AzureCredentials(val));
                }
                6 => {
                    let val = BedrockCredentials::decode(&field.value)?;
                    msg.credentials = Some(RequestedModelCredentials::BedrockCredentials(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RequestedModel_ModelParameterbytes {
    pub id: String,
    pub value: String,
}

impl RequestedModel_ModelParameterbytes {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.id));
        chunks.push(encode_string_field(2, &self.value));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.id = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.value = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AgentRunRequest {
    pub conversation_state: Option<ConversationStateStructure>,
    pub action: Option<ConversationAction>,
    pub model_details: Option<ModelDetails>,
    pub requested_model: Option<RequestedModel>,
    pub mcp_tools: Option<McpTools>,
    pub conversation_id: Option<String>,
    pub mcp_file_system_options: Option<McpFileSystemOptions>,
    pub skill_options: Option<SkillOptions>,
    pub custom_system_prompt: Option<String>,
}

impl AgentRunRequest {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.conversation_state {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.action {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        if let Some(ref val) = self.model_details {
            chunks.push(encode_message_field(3, &val.encode()));
        }
        if let Some(ref val) = self.requested_model {
            chunks.push(encode_message_field(9, &val.encode()));
        }
        if let Some(ref val) = self.mcp_tools {
            chunks.push(encode_message_field(4, &val.encode()));
        }
        if let Some(ref val) = self.conversation_id {
            chunks.push(encode_string_field_always(5, val));
        }
        if let Some(ref val) = self.mcp_file_system_options {
            chunks.push(encode_message_field(6, &val.encode()));
        }
        if let Some(ref val) = self.skill_options {
            chunks.push(encode_message_field(7, &val.encode()));
        }
        if let Some(ref val) = self.custom_system_prompt {
            chunks.push(encode_string_field_always(8, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.conversation_state =
                        Some(ConversationStateStructure::decode(&field.value)?);
                }
                2 => {
                    msg.action = Some(ConversationAction::decode(&field.value)?);
                }
                3 => {
                    msg.model_details = Some(ModelDetails::decode(&field.value)?);
                }
                9 => {
                    msg.requested_model = Some(RequestedModel::decode(&field.value)?);
                }
                4 => {
                    msg.mcp_tools = Some(McpTools::decode(&field.value)?);
                }
                5 => {
                    msg.conversation_id = Some(String::from_utf8(field.value).ok()?);
                }
                6 => {
                    msg.mcp_file_system_options = Some(McpFileSystemOptions::decode(&field.value)?);
                }
                7 => {
                    msg.skill_options = Some(SkillOptions::decode(&field.value)?);
                }
                8 => {
                    msg.custom_system_prompt = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TextDeltaUpdate {
    pub text: String,
}

impl TextDeltaUpdate {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.text));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.text = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ToolCallStartedUpdate {
    pub call_id: String,
    pub tool_call: Option<ToolCall>,
    pub model_call_id: String,
}

impl ToolCallStartedUpdate {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.call_id));
        if let Some(ref val) = self.tool_call {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        chunks.push(encode_string_field(3, &self.model_call_id));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.call_id = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.tool_call = Some(ToolCall::decode(&field.value)?);
                }
                3 => {
                    msg.model_call_id = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ToolCallCompletedUpdate {
    pub call_id: String,
    pub tool_call: Option<ToolCall>,
    pub model_call_id: String,
}

impl ToolCallCompletedUpdate {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.call_id));
        if let Some(ref val) = self.tool_call {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        chunks.push(encode_string_field(3, &self.model_call_id));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.call_id = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.tool_call = Some(ToolCall::decode(&field.value)?);
                }
                3 => {
                    msg.model_call_id = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ToolCallDeltaUpdate {
    pub call_id: String,
    pub tool_call_delta: Option<Box<ToolCallDelta>>,
    pub model_call_id: String,
}

impl ToolCallDeltaUpdate {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.call_id));
        if let Some(ref val) = self.tool_call_delta {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        chunks.push(encode_string_field(3, &self.model_call_id));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.call_id = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.tool_call_delta = Some(Box::new(ToolCallDelta::decode(&field.value)?));
                }
                3 => {
                    msg.model_call_id = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PartialToolCallUpdate {
    pub call_id: String,
    pub tool_call: Option<ToolCall>,
    pub args_text_delta: String,
    pub model_call_id: String,
}

impl PartialToolCallUpdate {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.call_id));
        if let Some(ref val) = self.tool_call {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        chunks.push(encode_string_field(3, &self.args_text_delta));
        chunks.push(encode_string_field(4, &self.model_call_id));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.call_id = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.tool_call = Some(ToolCall::decode(&field.value)?);
                }
                3 => {
                    msg.args_text_delta = String::from_utf8(field.value).ok()?;
                }
                4 => {
                    msg.model_call_id = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ThinkingDeltaUpdate {
    pub text: String,
}

impl ThinkingDeltaUpdate {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.text));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.text = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ThinkingCompletedUpdate {
    pub thinking_duration_ms: i32,
}

impl ThinkingCompletedUpdate {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.thinking_duration_ms != 0 {
            chunks.push(encode_varint_field_always(
                1,
                self.thinking_duration_ms as u64,
            ));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.thinking_duration_ms = val as i32;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TokenDeltaUpdate {
    pub tokens: i32,
}

impl TokenDeltaUpdate {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.tokens != 0 {
            chunks.push(encode_varint_field_always(1, self.tokens as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.tokens = val as i32;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SummaryUpdate {
    pub summary: String,
}

impl SummaryUpdate {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.summary));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.summary = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SummaryStartedUpdate {}

impl SummaryStartedUpdate {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct HeartbeatUpdate {}

impl HeartbeatUpdate {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SummaryCompletedUpdate {}

impl SummaryCompletedUpdate {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShellOutputDeltaUpdateEvent {
    Stdout(ShellStreamStdout),
    Stderr(ShellStreamStderr),
    Exit(ShellStreamExit),
    Start(ShellStreamStart),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShellOutputDeltaUpdate {
    pub event: Option<ShellOutputDeltaUpdateEvent>,
}

impl ShellOutputDeltaUpdate {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.event {
            match val {
                ShellOutputDeltaUpdateEvent::Stdout(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                ShellOutputDeltaUpdateEvent::Stderr(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                ShellOutputDeltaUpdateEvent::Exit(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
                ShellOutputDeltaUpdateEvent::Start(ref inner) => {
                    chunks.push(encode_message_field(4, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = ShellStreamStdout::decode(&field.value)?;
                    msg.event = Some(ShellOutputDeltaUpdateEvent::Stdout(val));
                }
                2 => {
                    let val = ShellStreamStderr::decode(&field.value)?;
                    msg.event = Some(ShellOutputDeltaUpdateEvent::Stderr(val));
                }
                3 => {
                    let val = ShellStreamExit::decode(&field.value)?;
                    msg.event = Some(ShellOutputDeltaUpdateEvent::Exit(val));
                }
                4 => {
                    let val = ShellStreamStart::decode(&field.value)?;
                    msg.event = Some(ShellOutputDeltaUpdateEvent::Start(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TurnEndedUpdate {}

impl TurnEndedUpdate {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct UserMessageAppendedUpdate {
    pub user_message: Option<UserMessage>,
}

impl UserMessageAppendedUpdate {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.user_message {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.user_message = Some(UserMessage::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct StepStartedUpdate {
    pub step_id: u64,
}

impl StepStartedUpdate {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.step_id != 0 {
            chunks.push(encode_varint_field_always(1, self.step_id as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.step_id = val as u64;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct StepCompletedUpdate {
    pub step_id: u64,
    pub step_duration_ms: i64,
}

impl StepCompletedUpdate {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.step_id != 0 {
            chunks.push(encode_varint_field_always(1, self.step_id as u64));
        }
        if self.step_duration_ms != 0 {
            chunks.push(encode_varint_field_always(2, self.step_duration_ms as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.step_id = val as u64;
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.step_duration_ms = val as i64;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InteractionUpdateMessage {
    TextDelta(TextDeltaUpdate),
    PartialToolCall(PartialToolCallUpdate),
    ToolCallDelta(Box<ToolCallDeltaUpdate>),
    ToolCallStarted(ToolCallStartedUpdate),
    ToolCallCompleted(ToolCallCompletedUpdate),
    ThinkingDelta(ThinkingDeltaUpdate),
    ThinkingCompleted(ThinkingCompletedUpdate),
    UserMessageAppended(UserMessageAppendedUpdate),
    TokenDelta(TokenDeltaUpdate),
    Summary(SummaryUpdate),
    SummaryStarted(SummaryStartedUpdate),
    SummaryCompleted(SummaryCompletedUpdate),
    ShellOutputDelta(ShellOutputDeltaUpdate),
    Heartbeat(HeartbeatUpdate),
    TurnEnded(TurnEndedUpdate),
    StepStarted(StepStartedUpdate),
    StepCompleted(StepCompletedUpdate),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct InteractionUpdate {
    pub message: Option<InteractionUpdateMessage>,
}

impl InteractionUpdate {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.message {
            match val {
                InteractionUpdateMessage::TextDelta(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                InteractionUpdateMessage::PartialToolCall(ref inner) => {
                    chunks.push(encode_message_field(7, &inner.encode()));
                }
                InteractionUpdateMessage::ToolCallDelta(ref inner) => {
                    chunks.push(encode_message_field(15, &inner.encode()));
                }
                InteractionUpdateMessage::ToolCallStarted(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                InteractionUpdateMessage::ToolCallCompleted(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
                InteractionUpdateMessage::ThinkingDelta(ref inner) => {
                    chunks.push(encode_message_field(4, &inner.encode()));
                }
                InteractionUpdateMessage::ThinkingCompleted(ref inner) => {
                    chunks.push(encode_message_field(5, &inner.encode()));
                }
                InteractionUpdateMessage::UserMessageAppended(ref inner) => {
                    chunks.push(encode_message_field(6, &inner.encode()));
                }
                InteractionUpdateMessage::TokenDelta(ref inner) => {
                    chunks.push(encode_message_field(8, &inner.encode()));
                }
                InteractionUpdateMessage::Summary(ref inner) => {
                    chunks.push(encode_message_field(9, &inner.encode()));
                }
                InteractionUpdateMessage::SummaryStarted(ref inner) => {
                    chunks.push(encode_message_field(10, &inner.encode()));
                }
                InteractionUpdateMessage::SummaryCompleted(ref inner) => {
                    chunks.push(encode_message_field(11, &inner.encode()));
                }
                InteractionUpdateMessage::ShellOutputDelta(ref inner) => {
                    chunks.push(encode_message_field(12, &inner.encode()));
                }
                InteractionUpdateMessage::Heartbeat(ref inner) => {
                    chunks.push(encode_message_field(13, &inner.encode()));
                }
                InteractionUpdateMessage::TurnEnded(ref inner) => {
                    chunks.push(encode_message_field(14, &inner.encode()));
                }
                InteractionUpdateMessage::StepStarted(ref inner) => {
                    chunks.push(encode_message_field(16, &inner.encode()));
                }
                InteractionUpdateMessage::StepCompleted(ref inner) => {
                    chunks.push(encode_message_field(17, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = TextDeltaUpdate::decode(&field.value)?;
                    msg.message = Some(InteractionUpdateMessage::TextDelta(val));
                }
                7 => {
                    let val = PartialToolCallUpdate::decode(&field.value)?;
                    msg.message = Some(InteractionUpdateMessage::PartialToolCall(val));
                }
                15 => {
                    let val = Box::new(ToolCallDeltaUpdate::decode(&field.value)?);
                    msg.message = Some(InteractionUpdateMessage::ToolCallDelta(val));
                }
                2 => {
                    let val = ToolCallStartedUpdate::decode(&field.value)?;
                    msg.message = Some(InteractionUpdateMessage::ToolCallStarted(val));
                }
                3 => {
                    let val = ToolCallCompletedUpdate::decode(&field.value)?;
                    msg.message = Some(InteractionUpdateMessage::ToolCallCompleted(val));
                }
                4 => {
                    let val = ThinkingDeltaUpdate::decode(&field.value)?;
                    msg.message = Some(InteractionUpdateMessage::ThinkingDelta(val));
                }
                5 => {
                    let val = ThinkingCompletedUpdate::decode(&field.value)?;
                    msg.message = Some(InteractionUpdateMessage::ThinkingCompleted(val));
                }
                6 => {
                    let val = UserMessageAppendedUpdate::decode(&field.value)?;
                    msg.message = Some(InteractionUpdateMessage::UserMessageAppended(val));
                }
                8 => {
                    let val = TokenDeltaUpdate::decode(&field.value)?;
                    msg.message = Some(InteractionUpdateMessage::TokenDelta(val));
                }
                9 => {
                    let val = SummaryUpdate::decode(&field.value)?;
                    msg.message = Some(InteractionUpdateMessage::Summary(val));
                }
                10 => {
                    let val = SummaryStartedUpdate::decode(&field.value)?;
                    msg.message = Some(InteractionUpdateMessage::SummaryStarted(val));
                }
                11 => {
                    let val = SummaryCompletedUpdate::decode(&field.value)?;
                    msg.message = Some(InteractionUpdateMessage::SummaryCompleted(val));
                }
                12 => {
                    let val = ShellOutputDeltaUpdate::decode(&field.value)?;
                    msg.message = Some(InteractionUpdateMessage::ShellOutputDelta(val));
                }
                13 => {
                    let val = HeartbeatUpdate::decode(&field.value)?;
                    msg.message = Some(InteractionUpdateMessage::Heartbeat(val));
                }
                14 => {
                    let val = TurnEndedUpdate::decode(&field.value)?;
                    msg.message = Some(InteractionUpdateMessage::TurnEnded(val));
                }
                16 => {
                    let val = StepStartedUpdate::decode(&field.value)?;
                    msg.message = Some(InteractionUpdateMessage::StepStarted(val));
                }
                17 => {
                    let val = StepCompletedUpdate::decode(&field.value)?;
                    msg.message = Some(InteractionUpdateMessage::StepCompleted(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InteractionQueryQuery {
    WebSearchRequestQuery(WebSearchRequestQuery),
    AskQuestionInteractionQuery(AskQuestionInteractionQuery),
    SwitchModeRequestQuery(SwitchModeRequestQuery),
    ExaSearchRequestQuery(ExaSearchRequestQuery),
    ExaFetchRequestQuery(ExaFetchRequestQuery),
    CreatePlanRequestQuery(CreatePlanRequestQuery),
    SetupVmEnvironmentArgs(SetupVmEnvironmentArgs),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct InteractionQuery {
    pub id: u32,
    pub query: Option<InteractionQueryQuery>,
}

impl InteractionQuery {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.id != 0 {
            chunks.push(encode_varint_field_always(1, self.id as u64));
        }
        if let Some(ref val) = self.query {
            match val {
                InteractionQueryQuery::WebSearchRequestQuery(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                InteractionQueryQuery::AskQuestionInteractionQuery(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
                InteractionQueryQuery::SwitchModeRequestQuery(ref inner) => {
                    chunks.push(encode_message_field(4, &inner.encode()));
                }
                InteractionQueryQuery::ExaSearchRequestQuery(ref inner) => {
                    chunks.push(encode_message_field(5, &inner.encode()));
                }
                InteractionQueryQuery::ExaFetchRequestQuery(ref inner) => {
                    chunks.push(encode_message_field(6, &inner.encode()));
                }
                InteractionQueryQuery::CreatePlanRequestQuery(ref inner) => {
                    chunks.push(encode_message_field(7, &inner.encode()));
                }
                InteractionQueryQuery::SetupVmEnvironmentArgs(ref inner) => {
                    chunks.push(encode_message_field(8, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.id = val as u32;
                }
                2 => {
                    let val = WebSearchRequestQuery::decode(&field.value)?;
                    msg.query = Some(InteractionQueryQuery::WebSearchRequestQuery(val));
                }
                3 => {
                    let val = AskQuestionInteractionQuery::decode(&field.value)?;
                    msg.query = Some(InteractionQueryQuery::AskQuestionInteractionQuery(val));
                }
                4 => {
                    let val = SwitchModeRequestQuery::decode(&field.value)?;
                    msg.query = Some(InteractionQueryQuery::SwitchModeRequestQuery(val));
                }
                5 => {
                    let val = ExaSearchRequestQuery::decode(&field.value)?;
                    msg.query = Some(InteractionQueryQuery::ExaSearchRequestQuery(val));
                }
                6 => {
                    let val = ExaFetchRequestQuery::decode(&field.value)?;
                    msg.query = Some(InteractionQueryQuery::ExaFetchRequestQuery(val));
                }
                7 => {
                    let val = CreatePlanRequestQuery::decode(&field.value)?;
                    msg.query = Some(InteractionQueryQuery::CreatePlanRequestQuery(val));
                }
                8 => {
                    let val = SetupVmEnvironmentArgs::decode(&field.value)?;
                    msg.query = Some(InteractionQueryQuery::SetupVmEnvironmentArgs(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InteractionResponseResult {
    WebSearchRequestResponse(WebSearchRequestResponse),
    AskQuestionInteractionResponse(AskQuestionInteractionResponse),
    SwitchModeRequestResponse(SwitchModeRequestResponse),
    ExaSearchRequestResponse(ExaSearchRequestResponse),
    ExaFetchRequestResponse(ExaFetchRequestResponse),
    CreatePlanRequestResponse(CreatePlanRequestResponse),
    SetupVmEnvironmentResult(SetupVmEnvironmentResult),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct InteractionResponse {
    pub id: u32,
    pub result: Option<InteractionResponseResult>,
}

impl InteractionResponse {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.id != 0 {
            chunks.push(encode_varint_field_always(1, self.id as u64));
        }
        if let Some(ref val) = self.result {
            match val {
                InteractionResponseResult::WebSearchRequestResponse(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                InteractionResponseResult::AskQuestionInteractionResponse(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
                InteractionResponseResult::SwitchModeRequestResponse(ref inner) => {
                    chunks.push(encode_message_field(4, &inner.encode()));
                }
                InteractionResponseResult::ExaSearchRequestResponse(ref inner) => {
                    chunks.push(encode_message_field(5, &inner.encode()));
                }
                InteractionResponseResult::ExaFetchRequestResponse(ref inner) => {
                    chunks.push(encode_message_field(6, &inner.encode()));
                }
                InteractionResponseResult::CreatePlanRequestResponse(ref inner) => {
                    chunks.push(encode_message_field(7, &inner.encode()));
                }
                InteractionResponseResult::SetupVmEnvironmentResult(ref inner) => {
                    chunks.push(encode_message_field(8, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.id = val as u32;
                }
                2 => {
                    let val = WebSearchRequestResponse::decode(&field.value)?;
                    msg.result = Some(InteractionResponseResult::WebSearchRequestResponse(val));
                }
                3 => {
                    let val = AskQuestionInteractionResponse::decode(&field.value)?;
                    msg.result = Some(InteractionResponseResult::AskQuestionInteractionResponse(
                        val,
                    ));
                }
                4 => {
                    let val = SwitchModeRequestResponse::decode(&field.value)?;
                    msg.result = Some(InteractionResponseResult::SwitchModeRequestResponse(val));
                }
                5 => {
                    let val = ExaSearchRequestResponse::decode(&field.value)?;
                    msg.result = Some(InteractionResponseResult::ExaSearchRequestResponse(val));
                }
                6 => {
                    let val = ExaFetchRequestResponse::decode(&field.value)?;
                    msg.result = Some(InteractionResponseResult::ExaFetchRequestResponse(val));
                }
                7 => {
                    let val = CreatePlanRequestResponse::decode(&field.value)?;
                    msg.result = Some(InteractionResponseResult::CreatePlanRequestResponse(val));
                }
                8 => {
                    let val = SetupVmEnvironmentResult::decode(&field.value)?;
                    msg.result = Some(InteractionResponseResult::SetupVmEnvironmentResult(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AskQuestionInteractionQuery {
    pub args: Option<AskQuestionArgs>,
    pub tool_call_id: String,
}

impl AskQuestionInteractionQuery {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        chunks.push(encode_string_field(2, &self.tool_call_id));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(AskQuestionArgs::decode(&field.value)?);
                }
                2 => {
                    msg.tool_call_id = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AskQuestionInteractionResponse {
    pub result: Option<AskQuestionResult>,
}

impl AskQuestionInteractionResponse {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.result = Some(AskQuestionResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ClientHeartbeat {}

impl ClientHeartbeat {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PrewarmRequest {
    pub model_details: Option<ModelDetails>,
    pub requested_model: Option<RequestedModel>,
    pub conversation_id: Option<String>,
    pub conversation_state: Option<ConversationStateStructure>,
    pub mcp_tools: Option<McpTools>,
    pub mcp_file_system_options: Option<McpFileSystemOptions>,
    pub best_of_n_group_id: Option<String>,
    pub try_use_best_of_n_promotion: Option<bool>,
    pub custom_system_prompt: Option<String>,
}

impl PrewarmRequest {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.model_details {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.requested_model {
            chunks.push(encode_message_field(9, &val.encode()));
        }
        if let Some(ref val) = self.conversation_id {
            chunks.push(encode_string_field_always(2, val));
        }
        if let Some(ref val) = self.conversation_state {
            chunks.push(encode_message_field(3, &val.encode()));
        }
        if let Some(ref val) = self.mcp_tools {
            chunks.push(encode_message_field(4, &val.encode()));
        }
        if let Some(ref val) = self.mcp_file_system_options {
            chunks.push(encode_message_field(5, &val.encode()));
        }
        if let Some(ref val) = self.best_of_n_group_id {
            chunks.push(encode_string_field_always(6, val));
        }
        if let Some(ref val) = self.try_use_best_of_n_promotion {
            chunks.push(encode_bool_field_always(7, *val));
        }
        if let Some(ref val) = self.custom_system_prompt {
            chunks.push(encode_string_field_always(8, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.model_details = Some(ModelDetails::decode(&field.value)?);
                }
                9 => {
                    msg.requested_model = Some(RequestedModel::decode(&field.value)?);
                }
                2 => {
                    msg.conversation_id = Some(String::from_utf8(field.value).ok()?);
                }
                3 => {
                    msg.conversation_state =
                        Some(ConversationStateStructure::decode(&field.value)?);
                }
                4 => {
                    msg.mcp_tools = Some(McpTools::decode(&field.value)?);
                }
                5 => {
                    msg.mcp_file_system_options = Some(McpFileSystemOptions::decode(&field.value)?);
                }
                6 => {
                    msg.best_of_n_group_id = Some(String::from_utf8(field.value).ok()?);
                }
                7 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.try_use_best_of_n_promotion = Some(val != 0);
                }
                8 => {
                    msg.custom_system_prompt = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExecServerAbort {
    pub id: u32,
}

impl ExecServerAbort {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.id != 0 {
            chunks.push(encode_varint_field_always(1, self.id as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.id = val as u32;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecServerControlMessageMessage {
    Abort(ExecServerAbort),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExecServerControlMessage {
    pub message: Option<ExecServerControlMessageMessage>,
}

impl ExecServerControlMessage {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.message {
            match val {
                ExecServerControlMessageMessage::Abort(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = ExecServerAbort::decode(&field.value)?;
                    msg.message = Some(ExecServerControlMessageMessage::Abort(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AgentClientMessageMessage {
    RunRequest(AgentRunRequest),
    ExecClientMessage(ExecClientMessage),
    ExecClientControlMessage(ExecClientControlMessage),
    KvClientMessage(KvClientMessage),
    ConversationAction(ConversationAction),
    InteractionResponse(InteractionResponse),
    ClientHeartbeat(ClientHeartbeat),
    PrewarmRequest(PrewarmRequest),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AgentClientMessage {
    pub message: Option<AgentClientMessageMessage>,
}

impl AgentClientMessage {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.message {
            match val {
                AgentClientMessageMessage::RunRequest(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                AgentClientMessageMessage::ExecClientMessage(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                AgentClientMessageMessage::ExecClientControlMessage(ref inner) => {
                    chunks.push(encode_message_field(5, &inner.encode()));
                }
                AgentClientMessageMessage::KvClientMessage(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
                AgentClientMessageMessage::ConversationAction(ref inner) => {
                    chunks.push(encode_message_field(4, &inner.encode()));
                }
                AgentClientMessageMessage::InteractionResponse(ref inner) => {
                    chunks.push(encode_message_field(6, &inner.encode()));
                }
                AgentClientMessageMessage::ClientHeartbeat(ref inner) => {
                    chunks.push(encode_message_field(7, &inner.encode()));
                }
                AgentClientMessageMessage::PrewarmRequest(ref inner) => {
                    chunks.push(encode_message_field(8, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = AgentRunRequest::decode(&field.value)?;
                    msg.message = Some(AgentClientMessageMessage::RunRequest(val));
                }
                2 => {
                    let val = ExecClientMessage::decode(&field.value)?;
                    msg.message = Some(AgentClientMessageMessage::ExecClientMessage(val));
                }
                5 => {
                    let val = ExecClientControlMessage::decode(&field.value)?;
                    msg.message = Some(AgentClientMessageMessage::ExecClientControlMessage(val));
                }
                3 => {
                    let val = KvClientMessage::decode(&field.value)?;
                    msg.message = Some(AgentClientMessageMessage::KvClientMessage(val));
                }
                4 => {
                    let val = ConversationAction::decode(&field.value)?;
                    msg.message = Some(AgentClientMessageMessage::ConversationAction(val));
                }
                6 => {
                    let val = InteractionResponse::decode(&field.value)?;
                    msg.message = Some(AgentClientMessageMessage::InteractionResponse(val));
                }
                7 => {
                    let val = ClientHeartbeat::decode(&field.value)?;
                    msg.message = Some(AgentClientMessageMessage::ClientHeartbeat(val));
                }
                8 => {
                    let val = PrewarmRequest::decode(&field.value)?;
                    msg.message = Some(AgentClientMessageMessage::PrewarmRequest(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AgentServerMessageMessage {
    InteractionUpdate(InteractionUpdate),
    ExecServerMessage(ExecServerMessage),
    ExecServerControlMessage(ExecServerControlMessage),
    ConversationCheckpointUpdate(ConversationStateStructure),
    KvServerMessage(KvServerMessage),
    InteractionQuery(InteractionQuery),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AgentServerMessage {
    pub message: Option<AgentServerMessageMessage>,
}

impl AgentServerMessage {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.message {
            match val {
                AgentServerMessageMessage::InteractionUpdate(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                AgentServerMessageMessage::ExecServerMessage(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                AgentServerMessageMessage::ExecServerControlMessage(ref inner) => {
                    chunks.push(encode_message_field(5, &inner.encode()));
                }
                AgentServerMessageMessage::ConversationCheckpointUpdate(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
                AgentServerMessageMessage::KvServerMessage(ref inner) => {
                    chunks.push(encode_message_field(4, &inner.encode()));
                }
                AgentServerMessageMessage::InteractionQuery(ref inner) => {
                    chunks.push(encode_message_field(7, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = InteractionUpdate::decode(&field.value)?;
                    msg.message = Some(AgentServerMessageMessage::InteractionUpdate(val));
                }
                2 => {
                    let val = ExecServerMessage::decode(&field.value)?;
                    msg.message = Some(AgentServerMessageMessage::ExecServerMessage(val));
                }
                5 => {
                    let val = ExecServerControlMessage::decode(&field.value)?;
                    msg.message = Some(AgentServerMessageMessage::ExecServerControlMessage(val));
                }
                3 => {
                    let val = ConversationStateStructure::decode(&field.value)?;
                    msg.message =
                        Some(AgentServerMessageMessage::ConversationCheckpointUpdate(val));
                }
                4 => {
                    let val = KvServerMessage::decode(&field.value)?;
                    msg.message = Some(AgentServerMessageMessage::KvServerMessage(val));
                }
                7 => {
                    let val = InteractionQuery::decode(&field.value)?;
                    msg.message = Some(AgentServerMessageMessage::InteractionQuery(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct NameAgentRequest {
    pub user_message: String,
}

impl NameAgentRequest {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.user_message));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.user_message = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct NameAgentResponse {
    pub name: String,
}

impl NameAgentResponse {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.name));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.name = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GetUsableModelsRequest {
    pub custom_model_ids: Vec<String>,
}

impl GetUsableModelsRequest {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_repeated_string_field(1, &self.custom_model_ids));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.custom_model_ids
                        .push(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GetUsableModelsResponse {
    pub models: Vec<ModelDetails>,
}

impl GetUsableModelsResponse {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        let items_models: Vec<Vec<u8>> = self.models.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(1, &items_models));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.models.push(ModelDetails::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GetDefaultModelForCliRequest {}

impl GetDefaultModelForCliRequest {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GetDefaultModelForCliResponse {
    pub model: Option<ModelDetails>,
}

impl GetDefaultModelForCliResponse {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.model {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.model = Some(ModelDetails::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GetAllowedModelIntentsRequest {}

impl GetAllowedModelIntentsRequest {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GetAllowedModelIntentsResponse {
    pub model_intents: Vec<String>,
}

impl GetAllowedModelIntentsResponse {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_repeated_string_field(1, &self.model_intents));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.model_intents.push(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct IdeEditorsStateFile {
    pub relative_path: String,
    pub absolute_path: String,
    pub is_currently_focused: Option<bool>,
    pub current_line_number: Option<i32>,
    pub current_line_text: Option<String>,
    pub line_count: Option<i32>,
}

impl IdeEditorsStateFile {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.relative_path));
        chunks.push(encode_string_field(2, &self.absolute_path));
        if let Some(ref val) = self.is_currently_focused {
            chunks.push(encode_bool_field_always(3, *val));
        }
        if let Some(ref val) = self.current_line_number {
            chunks.push(encode_varint_field_always(4, *val as u64));
        }
        if let Some(ref val) = self.current_line_text {
            chunks.push(encode_string_field_always(5, val));
        }
        if let Some(ref val) = self.line_count {
            chunks.push(encode_varint_field_always(6, *val as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.relative_path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.absolute_path = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.is_currently_focused = Some(val != 0);
                }
                4 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.current_line_number = Some(val as i32);
                }
                5 => {
                    msg.current_line_text = Some(String::from_utf8(field.value).ok()?);
                }
                6 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.line_count = Some(val as i32);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct IdeEditorsStateLite {
    pub recently_viewed_files: Vec<IdeEditorsStateFile>,
}

impl IdeEditorsStateLite {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        let items_recently_viewed_files: Vec<Vec<u8>> = self
            .recently_viewed_files
            .iter()
            .map(|item| item.encode())
            .collect();
        chunks.push(encode_repeated_message_field(
            1,
            &items_recently_viewed_files,
        ));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.recently_viewed_files
                        .push(IdeEditorsStateFile::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ApplyAgentDiffToolCall {
    pub args: Option<ApplyAgentDiffArgs>,
    pub result: Option<ApplyAgentDiffResult>,
}

impl ApplyAgentDiffToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(ApplyAgentDiffArgs::decode(&field.value)?);
                }
                2 => {
                    msg.result = Some(ApplyAgentDiffResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ApplyAgentDiffArgs {
    pub agent_id: String,
}

impl ApplyAgentDiffArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.agent_id));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.agent_id = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ApplyAgentDiffResultResult {
    Success(ApplyAgentDiffSuccess),
    Error(ApplyAgentDiffError),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ApplyAgentDiffResult {
    pub result: Option<ApplyAgentDiffResultResult>,
}

impl ApplyAgentDiffResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                ApplyAgentDiffResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                ApplyAgentDiffResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = ApplyAgentDiffSuccess::decode(&field.value)?;
                    msg.result = Some(ApplyAgentDiffResultResult::Success(val));
                }
                2 => {
                    let val = ApplyAgentDiffError::decode(&field.value)?;
                    msg.result = Some(ApplyAgentDiffResultResult::Error(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ApplyAgentDiffSuccess {
    pub applied_changes: Vec<AppliedAgentChange>,
}

impl ApplyAgentDiffSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        let items_applied_changes: Vec<Vec<u8>> = self
            .applied_changes
            .iter()
            .map(|item| item.encode())
            .collect();
        chunks.push(encode_repeated_message_field(1, &items_applied_changes));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.applied_changes
                        .push(AppliedAgentChange::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AppliedAgentChange {
    pub path: String,
    pub change_type: i32,
    pub before_content: Option<String>,
    pub after_content: Option<String>,
    pub error: Option<String>,
    pub message_for_model: Option<String>,
}

impl AppliedAgentChange {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        if self.change_type != 0 {
            chunks.push(encode_varint_field_always(2, self.change_type as u64));
        }
        if let Some(ref val) = self.before_content {
            chunks.push(encode_string_field_always(3, val));
        }
        if let Some(ref val) = self.after_content {
            chunks.push(encode_string_field_always(4, val));
        }
        if let Some(ref val) = self.error {
            chunks.push(encode_string_field_always(5, val));
        }
        if let Some(ref val) = self.message_for_model {
            chunks.push(encode_string_field_always(6, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.change_type = val as i32;
                }
                3 => {
                    msg.before_content = Some(String::from_utf8(field.value).ok()?);
                }
                4 => {
                    msg.after_content = Some(String::from_utf8(field.value).ok()?);
                }
                5 => {
                    msg.error = Some(String::from_utf8(field.value).ok()?);
                }
                6 => {
                    msg.message_for_model = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ApplyAgentDiffError {
    pub error: String,
    pub applied_changes: Vec<AppliedAgentChange>,
}

impl ApplyAgentDiffError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.error));
        let items_applied_changes: Vec<Vec<u8>> = self
            .applied_changes
            .iter()
            .map(|item| item.encode())
            .collect();
        chunks.push(encode_repeated_message_field(2, &items_applied_changes));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.applied_changes
                        .push(AppliedAgentChange::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AskQuestionToolCall {
    pub args: Option<AskQuestionArgs>,
    pub result: Option<AskQuestionResult>,
}

impl AskQuestionToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(AskQuestionArgs::decode(&field.value)?);
                }
                2 => {
                    msg.result = Some(AskQuestionResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AskQuestionArgs {
    pub title: String,
    pub questions: Vec<AskQuestionArgs_Question>,
    pub run_async: bool,
    pub async_original_tool_call_id: String,
}

impl AskQuestionArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.title));
        let items_questions: Vec<Vec<u8>> =
            self.questions.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(2, &items_questions));
        chunks.push(encode_bool_field(5, self.run_async));
        chunks.push(encode_string_field(6, &self.async_original_tool_call_id));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.title = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.questions
                        .push(AskQuestionArgs_Question::decode(&field.value)?);
                }
                5 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.run_async = val != 0;
                }
                6 => {
                    msg.async_original_tool_call_id = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AskQuestionArgs_Question {
    pub id: String,
    pub prompt: String,
    pub options: Vec<AskQuestionArgs_Option>,
    pub allow_multiple: bool,
}

impl AskQuestionArgs_Question {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.id));
        chunks.push(encode_string_field(2, &self.prompt));
        let items_options: Vec<Vec<u8>> = self.options.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(3, &items_options));
        chunks.push(encode_bool_field(4, self.allow_multiple));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.id = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.prompt = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.options
                        .push(AskQuestionArgs_Option::decode(&field.value)?);
                }
                4 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.allow_multiple = val != 0;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AskQuestionArgs_Option {
    pub id: String,
    pub label: String,
}

impl AskQuestionArgs_Option {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.id));
        chunks.push(encode_string_field(2, &self.label));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.id = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.label = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AskQuestionAsync {}

impl AskQuestionAsync {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AskQuestionResultResult {
    Success(AskQuestionSuccess),
    Error(AskQuestionError),
    Rejected(AskQuestionRejected),
    Async(AskQuestionAsync),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AskQuestionResult {
    pub result: Option<AskQuestionResultResult>,
}

impl AskQuestionResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                AskQuestionResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                AskQuestionResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                AskQuestionResultResult::Rejected(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
                AskQuestionResultResult::Async(ref inner) => {
                    chunks.push(encode_message_field(4, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = AskQuestionSuccess::decode(&field.value)?;
                    msg.result = Some(AskQuestionResultResult::Success(val));
                }
                2 => {
                    let val = AskQuestionError::decode(&field.value)?;
                    msg.result = Some(AskQuestionResultResult::Error(val));
                }
                3 => {
                    let val = AskQuestionRejected::decode(&field.value)?;
                    msg.result = Some(AskQuestionResultResult::Rejected(val));
                }
                4 => {
                    let val = AskQuestionAsync::decode(&field.value)?;
                    msg.result = Some(AskQuestionResultResult::Async(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AskQuestionSuccess {
    pub answers: Vec<AskQuestionSuccess_Answer>,
}

impl AskQuestionSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        let items_answers: Vec<Vec<u8>> = self.answers.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(1, &items_answers));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.answers
                        .push(AskQuestionSuccess_Answer::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AskQuestionSuccess_Answer {
    pub question_id: String,
    pub selected_option_ids: Vec<String>,
}

impl AskQuestionSuccess_Answer {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.question_id));
        chunks.push(encode_repeated_string_field(2, &self.selected_option_ids));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.question_id = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.selected_option_ids
                        .push(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AskQuestionError {
    pub error_message: String,
}

impl AskQuestionError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.error_message));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.error_message = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AskQuestionRejected {
    pub reason: String,
}

impl AskQuestionRejected {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.reason));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.reason = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BackgroundShellSpawnArgs {
    pub command: String,
    pub working_directory: String,
    pub tool_call_id: String,
    pub parsing_result: Option<ShellCommandParsingResult>,
    pub sandbox_policy: Option<SandboxPolicy>,
    pub enable_write_shell_stdin_tool: bool,
}

impl BackgroundShellSpawnArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.command));
        chunks.push(encode_string_field(2, &self.working_directory));
        chunks.push(encode_string_field(3, &self.tool_call_id));
        if let Some(ref val) = self.parsing_result {
            chunks.push(encode_message_field(4, &val.encode()));
        }
        if let Some(ref val) = self.sandbox_policy {
            chunks.push(encode_message_field(5, &val.encode()));
        }
        chunks.push(encode_bool_field(6, self.enable_write_shell_stdin_tool));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.command = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.working_directory = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.tool_call_id = String::from_utf8(field.value).ok()?;
                }
                4 => {
                    msg.parsing_result = Some(ShellCommandParsingResult::decode(&field.value)?);
                }
                5 => {
                    msg.sandbox_policy = Some(SandboxPolicy::decode(&field.value)?);
                }
                6 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.enable_write_shell_stdin_tool = val != 0;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BackgroundShellSpawnResultResult {
    Success(BackgroundShellSpawnSuccess),
    Error(BackgroundShellSpawnError),
    Rejected(ShellRejected),
    PermissionDenied(ShellPermissionDenied),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BackgroundShellSpawnResult {
    pub result: Option<BackgroundShellSpawnResultResult>,
}

impl BackgroundShellSpawnResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                BackgroundShellSpawnResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                BackgroundShellSpawnResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                BackgroundShellSpawnResultResult::Rejected(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
                BackgroundShellSpawnResultResult::PermissionDenied(ref inner) => {
                    chunks.push(encode_message_field(4, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = BackgroundShellSpawnSuccess::decode(&field.value)?;
                    msg.result = Some(BackgroundShellSpawnResultResult::Success(val));
                }
                2 => {
                    let val = BackgroundShellSpawnError::decode(&field.value)?;
                    msg.result = Some(BackgroundShellSpawnResultResult::Error(val));
                }
                3 => {
                    let val = ShellRejected::decode(&field.value)?;
                    msg.result = Some(BackgroundShellSpawnResultResult::Rejected(val));
                }
                4 => {
                    let val = ShellPermissionDenied::decode(&field.value)?;
                    msg.result = Some(BackgroundShellSpawnResultResult::PermissionDenied(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BackgroundShellSpawnSuccess {
    pub shell_id: u32,
    pub command: String,
    pub working_directory: String,
    pub pid: Option<u32>,
}

impl BackgroundShellSpawnSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.shell_id != 0 {
            chunks.push(encode_varint_field_always(1, self.shell_id as u64));
        }
        chunks.push(encode_string_field(2, &self.command));
        chunks.push(encode_string_field(3, &self.working_directory));
        if let Some(ref val) = self.pid {
            chunks.push(encode_varint_field_always(4, *val as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.shell_id = val as u32;
                }
                2 => {
                    msg.command = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.working_directory = String::from_utf8(field.value).ok()?;
                }
                4 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.pid = Some(val as u32);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BackgroundShellSpawnError {
    pub command: String,
    pub working_directory: String,
    pub error: String,
}

impl BackgroundShellSpawnError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.command));
        chunks.push(encode_string_field(2, &self.working_directory));
        chunks.push(encode_string_field(3, &self.error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.command = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.working_directory = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WriteShellStdinArgs {
    pub shell_id: u32,
    pub chars: String,
}

impl WriteShellStdinArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.shell_id != 0 {
            chunks.push(encode_varint_field_always(1, self.shell_id as u64));
        }
        chunks.push(encode_string_field(2, &self.chars));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.shell_id = val as u32;
                }
                2 => {
                    msg.chars = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WriteShellStdinResultResult {
    Success(WriteShellStdinSuccess),
    Error(WriteShellStdinError),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WriteShellStdinResult {
    pub result: Option<WriteShellStdinResultResult>,
}

impl WriteShellStdinResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                WriteShellStdinResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                WriteShellStdinResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = WriteShellStdinSuccess::decode(&field.value)?;
                    msg.result = Some(WriteShellStdinResultResult::Success(val));
                }
                2 => {
                    let val = WriteShellStdinError::decode(&field.value)?;
                    msg.result = Some(WriteShellStdinResultResult::Error(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WriteShellStdinSuccess {
    pub shell_id: u32,
    pub terminal_file_length_before_input_written: u32,
}

impl WriteShellStdinSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.shell_id != 0 {
            chunks.push(encode_varint_field_always(1, self.shell_id as u64));
        }
        if self.terminal_file_length_before_input_written != 0 {
            chunks.push(encode_varint_field_always(
                2,
                self.terminal_file_length_before_input_written as u64,
            ));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.shell_id = val as u32;
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.terminal_file_length_before_input_written = val as u32;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WriteShellStdinError {
    pub error: String,
}

impl WriteShellStdinError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Coordinate {
    pub x: i32,
    pub y: i32,
}

impl Coordinate {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.x != 0 {
            chunks.push(encode_varint_field_always(1, self.x as u64));
        }
        if self.y != 0 {
            chunks.push(encode_varint_field_always(2, self.y as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.x = val as i32;
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.y = val as i32;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ComputerUseArgs {
    pub tool_call_id: String,
    pub actions: Vec<ComputerUseAction>,
}

impl ComputerUseArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.tool_call_id));
        let items_actions: Vec<Vec<u8>> = self.actions.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(2, &items_actions));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.tool_call_id = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.actions.push(ComputerUseAction::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComputerUseActionAction {
    MouseMove(MouseMoveAction),
    Click(ClickAction),
    MouseDown(MouseDownAction),
    MouseUp(MouseUpAction),
    Drag(DragAction),
    Scroll(ScrollAction),
    Type(TypeAction),
    Key(KeyAction),
    Wait(WaitAction),
    Screenshot(ScreenshotAction),
    CursorPosition(CursorPositionAction),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ComputerUseAction {
    pub action: Option<ComputerUseActionAction>,
}

impl ComputerUseAction {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.action {
            match val {
                ComputerUseActionAction::MouseMove(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                ComputerUseActionAction::Click(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                ComputerUseActionAction::MouseDown(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
                ComputerUseActionAction::MouseUp(ref inner) => {
                    chunks.push(encode_message_field(4, &inner.encode()));
                }
                ComputerUseActionAction::Drag(ref inner) => {
                    chunks.push(encode_message_field(5, &inner.encode()));
                }
                ComputerUseActionAction::Scroll(ref inner) => {
                    chunks.push(encode_message_field(6, &inner.encode()));
                }
                ComputerUseActionAction::Type(ref inner) => {
                    chunks.push(encode_message_field(7, &inner.encode()));
                }
                ComputerUseActionAction::Key(ref inner) => {
                    chunks.push(encode_message_field(8, &inner.encode()));
                }
                ComputerUseActionAction::Wait(ref inner) => {
                    chunks.push(encode_message_field(9, &inner.encode()));
                }
                ComputerUseActionAction::Screenshot(ref inner) => {
                    chunks.push(encode_message_field(10, &inner.encode()));
                }
                ComputerUseActionAction::CursorPosition(ref inner) => {
                    chunks.push(encode_message_field(11, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = MouseMoveAction::decode(&field.value)?;
                    msg.action = Some(ComputerUseActionAction::MouseMove(val));
                }
                2 => {
                    let val = ClickAction::decode(&field.value)?;
                    msg.action = Some(ComputerUseActionAction::Click(val));
                }
                3 => {
                    let val = MouseDownAction::decode(&field.value)?;
                    msg.action = Some(ComputerUseActionAction::MouseDown(val));
                }
                4 => {
                    let val = MouseUpAction::decode(&field.value)?;
                    msg.action = Some(ComputerUseActionAction::MouseUp(val));
                }
                5 => {
                    let val = DragAction::decode(&field.value)?;
                    msg.action = Some(ComputerUseActionAction::Drag(val));
                }
                6 => {
                    let val = ScrollAction::decode(&field.value)?;
                    msg.action = Some(ComputerUseActionAction::Scroll(val));
                }
                7 => {
                    let val = TypeAction::decode(&field.value)?;
                    msg.action = Some(ComputerUseActionAction::Type(val));
                }
                8 => {
                    let val = KeyAction::decode(&field.value)?;
                    msg.action = Some(ComputerUseActionAction::Key(val));
                }
                9 => {
                    let val = WaitAction::decode(&field.value)?;
                    msg.action = Some(ComputerUseActionAction::Wait(val));
                }
                10 => {
                    let val = ScreenshotAction::decode(&field.value)?;
                    msg.action = Some(ComputerUseActionAction::Screenshot(val));
                }
                11 => {
                    let val = CursorPositionAction::decode(&field.value)?;
                    msg.action = Some(ComputerUseActionAction::CursorPosition(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MouseMoveAction {
    pub coordinate: Option<Coordinate>,
}

impl MouseMoveAction {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.coordinate {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.coordinate = Some(Coordinate::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ClickAction {
    pub coordinate: Option<Coordinate>,
    pub button: i32,
    pub count: i32,
    pub modifier_keys: Option<String>,
}

impl ClickAction {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.coordinate {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if self.button != 0 {
            chunks.push(encode_varint_field_always(2, self.button as u64));
        }
        if self.count != 0 {
            chunks.push(encode_varint_field_always(3, self.count as u64));
        }
        if let Some(ref val) = self.modifier_keys {
            chunks.push(encode_string_field_always(4, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.coordinate = Some(Coordinate::decode(&field.value)?);
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.button = val as i32;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.count = val as i32;
                }
                4 => {
                    msg.modifier_keys = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MouseDownAction {
    pub button: i32,
}

impl MouseDownAction {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.button != 0 {
            chunks.push(encode_varint_field_always(1, self.button as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.button = val as i32;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MouseUpAction {
    pub button: i32,
}

impl MouseUpAction {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.button != 0 {
            chunks.push(encode_varint_field_always(1, self.button as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.button = val as i32;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DragAction {
    pub path: Vec<Coordinate>,
    pub button: i32,
}

impl DragAction {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        let items_path: Vec<Vec<u8>> = self.path.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(1, &items_path));
        if self.button != 0 {
            chunks.push(encode_varint_field_always(2, self.button as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path.push(Coordinate::decode(&field.value)?);
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.button = val as i32;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ScrollAction {
    pub coordinate: Option<Coordinate>,
    pub direction: i32,
    pub amount: i32,
    pub modifier_keys: Option<String>,
}

impl ScrollAction {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.coordinate {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if self.direction != 0 {
            chunks.push(encode_varint_field_always(2, self.direction as u64));
        }
        if self.amount != 0 {
            chunks.push(encode_varint_field_always(3, self.amount as u64));
        }
        if let Some(ref val) = self.modifier_keys {
            chunks.push(encode_string_field_always(4, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.coordinate = Some(Coordinate::decode(&field.value)?);
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.direction = val as i32;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.amount = val as i32;
                }
                4 => {
                    msg.modifier_keys = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TypeAction {
    pub text: String,
}

impl TypeAction {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.text));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.text = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct KeyAction {
    pub key: String,
    pub hold_duration_ms: Option<i32>,
}

impl KeyAction {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.key));
        if let Some(ref val) = self.hold_duration_ms {
            chunks.push(encode_varint_field_always(2, *val as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.key = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.hold_duration_ms = Some(val as i32);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WaitAction {
    pub duration_ms: i32,
}

impl WaitAction {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.duration_ms != 0 {
            chunks.push(encode_varint_field_always(1, self.duration_ms as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.duration_ms = val as i32;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ScreenshotAction {}

impl ScreenshotAction {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CursorPositionAction {}

impl CursorPositionAction {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComputerUseResultResult {
    Success(ComputerUseSuccess),
    Error(ComputerUseError),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ComputerUseResult {
    pub result: Option<ComputerUseResultResult>,
}

impl ComputerUseResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                ComputerUseResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                ComputerUseResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = ComputerUseSuccess::decode(&field.value)?;
                    msg.result = Some(ComputerUseResultResult::Success(val));
                }
                2 => {
                    let val = ComputerUseError::decode(&field.value)?;
                    msg.result = Some(ComputerUseResultResult::Error(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ComputerUseSuccess {
    pub action_count: i32,
    pub duration_ms: i32,
    pub screenshot: Option<String>,
    pub log: Option<String>,
    pub screenshot_path: Option<String>,
    pub cursor_position: Option<Coordinate>,
}

impl ComputerUseSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.action_count != 0 {
            chunks.push(encode_varint_field_always(1, self.action_count as u64));
        }
        if self.duration_ms != 0 {
            chunks.push(encode_varint_field_always(2, self.duration_ms as u64));
        }
        if let Some(ref val) = self.screenshot {
            chunks.push(encode_string_field_always(3, val));
        }
        if let Some(ref val) = self.log {
            chunks.push(encode_string_field_always(4, val));
        }
        if let Some(ref val) = self.screenshot_path {
            chunks.push(encode_string_field_always(5, val));
        }
        if let Some(ref val) = self.cursor_position {
            chunks.push(encode_message_field(6, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.action_count = val as i32;
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.duration_ms = val as i32;
                }
                3 => {
                    msg.screenshot = Some(String::from_utf8(field.value).ok()?);
                }
                4 => {
                    msg.log = Some(String::from_utf8(field.value).ok()?);
                }
                5 => {
                    msg.screenshot_path = Some(String::from_utf8(field.value).ok()?);
                }
                6 => {
                    msg.cursor_position = Some(Coordinate::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ComputerUseError {
    pub error: String,
    pub action_count: i32,
    pub duration_ms: i32,
    pub log: Option<String>,
    pub screenshot: Option<String>,
    pub screenshot_path: Option<String>,
}

impl ComputerUseError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.error));
        if self.action_count != 0 {
            chunks.push(encode_varint_field_always(2, self.action_count as u64));
        }
        if self.duration_ms != 0 {
            chunks.push(encode_varint_field_always(3, self.duration_ms as u64));
        }
        if let Some(ref val) = self.log {
            chunks.push(encode_string_field_always(4, val));
        }
        if let Some(ref val) = self.screenshot {
            chunks.push(encode_string_field_always(5, val));
        }
        if let Some(ref val) = self.screenshot_path {
            chunks.push(encode_string_field_always(6, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.action_count = val as i32;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.duration_ms = val as i32;
                }
                4 => {
                    msg.log = Some(String::from_utf8(field.value).ok()?);
                }
                5 => {
                    msg.screenshot = Some(String::from_utf8(field.value).ok()?);
                }
                6 => {
                    msg.screenshot_path = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ComputerUseToolCall {
    pub args: Option<ComputerUseArgs>,
    pub result: Option<ComputerUseResult>,
}

impl ComputerUseToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(ComputerUseArgs::decode(&field.value)?);
                }
                2 => {
                    msg.result = Some(ComputerUseResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CreatePlanToolCall {
    pub args: Option<CreatePlanArgs>,
    pub result: Option<CreatePlanResult>,
}

impl CreatePlanToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(CreatePlanArgs::decode(&field.value)?);
                }
                2 => {
                    msg.result = Some(CreatePlanResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Phase {
    pub name: String,
    pub todos: Vec<TodoItem>,
}

impl Phase {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.name));
        let items_todos: Vec<Vec<u8>> = self.todos.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(2, &items_todos));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.name = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.todos.push(TodoItem::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CreatePlanArgs {
    pub plan: String,
    pub todos: Vec<TodoItem>,
    pub overview: String,
    pub name: String,
    pub is_project: bool,
    pub phases: Vec<Phase>,
}

impl CreatePlanArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.plan));
        let items_todos: Vec<Vec<u8>> = self.todos.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(2, &items_todos));
        chunks.push(encode_string_field(3, &self.overview));
        chunks.push(encode_string_field(4, &self.name));
        chunks.push(encode_bool_field(5, self.is_project));
        let items_phases: Vec<Vec<u8>> = self.phases.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(6, &items_phases));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.plan = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.todos.push(TodoItem::decode(&field.value)?);
                }
                3 => {
                    msg.overview = String::from_utf8(field.value).ok()?;
                }
                4 => {
                    msg.name = String::from_utf8(field.value).ok()?;
                }
                5 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.is_project = val != 0;
                }
                6 => {
                    msg.phases.push(Phase::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CreatePlanResultResult {
    Success(CreatePlanSuccess),
    Error(CreatePlanError),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CreatePlanResult {
    pub plan_uri: String,
    pub result: Option<CreatePlanResultResult>,
}

impl CreatePlanResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(3, &self.plan_uri));
        if let Some(ref val) = self.result {
            match val {
                CreatePlanResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                CreatePlanResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                3 => {
                    msg.plan_uri = String::from_utf8(field.value).ok()?;
                }
                1 => {
                    let val = CreatePlanSuccess::decode(&field.value)?;
                    msg.result = Some(CreatePlanResultResult::Success(val));
                }
                2 => {
                    let val = CreatePlanError::decode(&field.value)?;
                    msg.result = Some(CreatePlanResultResult::Error(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CreatePlanSuccess {}

impl CreatePlanSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CreatePlanError {
    pub error: String,
}

impl CreatePlanError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CreatePlanRequestQuery {
    pub args: Option<CreatePlanArgs>,
    pub tool_call_id: String,
}

impl CreatePlanRequestQuery {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        chunks.push(encode_string_field(2, &self.tool_call_id));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(CreatePlanArgs::decode(&field.value)?);
                }
                2 => {
                    msg.tool_call_id = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CreatePlanRequestResponse {
    pub result: Option<CreatePlanResult>,
}

impl CreatePlanRequestResponse {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.result = Some(CreatePlanResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CursorRuleTypeGlobal {}

impl CursorRuleTypeGlobal {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CursorRuleTypeFileGlobs {
    pub globs: Vec<String>,
}

impl CursorRuleTypeFileGlobs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_repeated_string_field(1, &self.globs));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.globs.push(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CursorRuleTypeAgentFetched {
    pub description: String,
}

impl CursorRuleTypeAgentFetched {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.description));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.description = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CursorRuleTypeManuallyAttached {}

impl CursorRuleTypeManuallyAttached {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CursorRuleTypeType {
    Global(CursorRuleTypeGlobal),
    FileGlobbed(CursorRuleTypeFileGlobs),
    AgentFetched(CursorRuleTypeAgentFetched),
    ManuallyAttached(CursorRuleTypeManuallyAttached),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CursorRuleType {
    pub type_: Option<CursorRuleTypeType>,
}

impl CursorRuleType {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.type_ {
            match val {
                CursorRuleTypeType::Global(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                CursorRuleTypeType::FileGlobbed(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                CursorRuleTypeType::AgentFetched(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
                CursorRuleTypeType::ManuallyAttached(ref inner) => {
                    chunks.push(encode_message_field(4, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = CursorRuleTypeGlobal::decode(&field.value)?;
                    msg.type_ = Some(CursorRuleTypeType::Global(val));
                }
                2 => {
                    let val = CursorRuleTypeFileGlobs::decode(&field.value)?;
                    msg.type_ = Some(CursorRuleTypeType::FileGlobbed(val));
                }
                3 => {
                    let val = CursorRuleTypeAgentFetched::decode(&field.value)?;
                    msg.type_ = Some(CursorRuleTypeType::AgentFetched(val));
                }
                4 => {
                    let val = CursorRuleTypeManuallyAttached::decode(&field.value)?;
                    msg.type_ = Some(CursorRuleTypeType::ManuallyAttached(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CursorRule {
    pub full_path: String,
    pub content: String,
    pub type_: Option<CursorRuleType>,
    pub source: i32,
    pub git_remote_origin: Option<String>,
    pub parse_error: Option<String>,
}

impl CursorRule {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.full_path));
        chunks.push(encode_string_field(2, &self.content));
        if let Some(ref val) = self.type_ {
            chunks.push(encode_message_field(3, &val.encode()));
        }
        if self.source != 0 {
            chunks.push(encode_varint_field_always(4, self.source as u64));
        }
        if let Some(ref val) = self.git_remote_origin {
            chunks.push(encode_string_field_always(5, val));
        }
        if let Some(ref val) = self.parse_error {
            chunks.push(encode_string_field_always(6, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.full_path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.content = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.type_ = Some(CursorRuleType::decode(&field.value)?);
                }
                4 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.source = val as i32;
                }
                5 => {
                    msg.git_remote_origin = Some(String::from_utf8(field.value).ok()?);
                }
                6 => {
                    msg.parse_error = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DeleteArgs {
    pub path: String,
    pub tool_call_id: String,
}

impl DeleteArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        chunks.push(encode_string_field(2, &self.tool_call_id));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.tool_call_id = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeleteResultResult {
    Success(DeleteSuccess),
    FileNotFound(DeleteFileNotFound),
    NotFile(DeleteNotFile),
    PermissionDenied(DeletePermissionDenied),
    FileBusy(DeleteFileBusy),
    Rejected(DeleteRejected),
    Error(DeleteError),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DeleteResult {
    pub result: Option<DeleteResultResult>,
}

impl DeleteResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                DeleteResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                DeleteResultResult::FileNotFound(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                DeleteResultResult::NotFile(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
                DeleteResultResult::PermissionDenied(ref inner) => {
                    chunks.push(encode_message_field(4, &inner.encode()));
                }
                DeleteResultResult::FileBusy(ref inner) => {
                    chunks.push(encode_message_field(5, &inner.encode()));
                }
                DeleteResultResult::Rejected(ref inner) => {
                    chunks.push(encode_message_field(6, &inner.encode()));
                }
                DeleteResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(7, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = DeleteSuccess::decode(&field.value)?;
                    msg.result = Some(DeleteResultResult::Success(val));
                }
                2 => {
                    let val = DeleteFileNotFound::decode(&field.value)?;
                    msg.result = Some(DeleteResultResult::FileNotFound(val));
                }
                3 => {
                    let val = DeleteNotFile::decode(&field.value)?;
                    msg.result = Some(DeleteResultResult::NotFile(val));
                }
                4 => {
                    let val = DeletePermissionDenied::decode(&field.value)?;
                    msg.result = Some(DeleteResultResult::PermissionDenied(val));
                }
                5 => {
                    let val = DeleteFileBusy::decode(&field.value)?;
                    msg.result = Some(DeleteResultResult::FileBusy(val));
                }
                6 => {
                    let val = DeleteRejected::decode(&field.value)?;
                    msg.result = Some(DeleteResultResult::Rejected(val));
                }
                7 => {
                    let val = DeleteError::decode(&field.value)?;
                    msg.result = Some(DeleteResultResult::Error(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DeleteSuccess {
    pub path: String,
    pub deleted_file: String,
    pub file_size: i64,
    pub prev_content: String,
}

impl DeleteSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        chunks.push(encode_string_field(2, &self.deleted_file));
        if self.file_size != 0 {
            chunks.push(encode_varint_field_always(3, self.file_size as u64));
        }
        chunks.push(encode_string_field(4, &self.prev_content));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.deleted_file = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.file_size = val as i64;
                }
                4 => {
                    msg.prev_content = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DeleteFileNotFound {
    pub path: String,
}

impl DeleteFileNotFound {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DeleteNotFile {
    pub path: String,
    pub actual_type: String,
}

impl DeleteNotFile {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        chunks.push(encode_string_field(2, &self.actual_type));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.actual_type = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DeletePermissionDenied {
    pub path: String,
    pub client_visible_error: String,
    pub is_readonly: bool,
}

impl DeletePermissionDenied {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        chunks.push(encode_string_field(2, &self.client_visible_error));
        chunks.push(encode_bool_field(3, self.is_readonly));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.client_visible_error = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.is_readonly = val != 0;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DeleteFileBusy {
    pub path: String,
}

impl DeleteFileBusy {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DeleteRejected {
    pub path: String,
    pub reason: String,
}

impl DeleteRejected {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        chunks.push(encode_string_field(2, &self.reason));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.reason = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DeleteError {
    pub path: String,
    pub error: String,
}

impl DeleteError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        chunks.push(encode_string_field(2, &self.error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DeleteToolCall {
    pub args: Option<DeleteArgs>,
    pub result: Option<DeleteResult>,
}

impl DeleteToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(DeleteArgs::decode(&field.value)?);
                }
                2 => {
                    msg.result = Some(DeleteResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DiagnosticsArgs {
    pub path: String,
    pub tool_call_id: String,
}

impl DiagnosticsArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        chunks.push(encode_string_field(2, &self.tool_call_id));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.tool_call_id = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticsResultResult {
    Success(DiagnosticsSuccess),
    Error(DiagnosticsError),
    Rejected(DiagnosticsRejected),
    FileNotFound(DiagnosticsFileNotFound),
    PermissionDenied(DiagnosticsPermissionDenied),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DiagnosticsResult {
    pub result: Option<DiagnosticsResultResult>,
}

impl DiagnosticsResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                DiagnosticsResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                DiagnosticsResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                DiagnosticsResultResult::Rejected(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
                DiagnosticsResultResult::FileNotFound(ref inner) => {
                    chunks.push(encode_message_field(4, &inner.encode()));
                }
                DiagnosticsResultResult::PermissionDenied(ref inner) => {
                    chunks.push(encode_message_field(5, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = DiagnosticsSuccess::decode(&field.value)?;
                    msg.result = Some(DiagnosticsResultResult::Success(val));
                }
                2 => {
                    let val = DiagnosticsError::decode(&field.value)?;
                    msg.result = Some(DiagnosticsResultResult::Error(val));
                }
                3 => {
                    let val = DiagnosticsRejected::decode(&field.value)?;
                    msg.result = Some(DiagnosticsResultResult::Rejected(val));
                }
                4 => {
                    let val = DiagnosticsFileNotFound::decode(&field.value)?;
                    msg.result = Some(DiagnosticsResultResult::FileNotFound(val));
                }
                5 => {
                    let val = DiagnosticsPermissionDenied::decode(&field.value)?;
                    msg.result = Some(DiagnosticsResultResult::PermissionDenied(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DiagnosticsSuccess {
    pub path: String,
    pub diagnostics: Vec<Diagnostic>,
    pub total_diagnostics: i32,
}

impl DiagnosticsSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        let items_diagnostics: Vec<Vec<u8>> =
            self.diagnostics.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(2, &items_diagnostics));
        if self.total_diagnostics != 0 {
            chunks.push(encode_varint_field_always(3, self.total_diagnostics as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.diagnostics.push(Diagnostic::decode(&field.value)?);
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.total_diagnostics = val as i32;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Diagnostic {
    pub severity: i32,
    pub range: Option<Range>,
    pub message: String,
    pub source: String,
    pub code: String,
    pub is_stale: bool,
}

impl Diagnostic {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.severity != 0 {
            chunks.push(encode_varint_field_always(1, self.severity as u64));
        }
        if let Some(ref val) = self.range {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        chunks.push(encode_string_field(3, &self.message));
        chunks.push(encode_string_field(4, &self.source));
        chunks.push(encode_string_field(5, &self.code));
        chunks.push(encode_bool_field(6, self.is_stale));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.severity = val as i32;
                }
                2 => {
                    msg.range = Some(Range::decode(&field.value)?);
                }
                3 => {
                    msg.message = String::from_utf8(field.value).ok()?;
                }
                4 => {
                    msg.source = String::from_utf8(field.value).ok()?;
                }
                5 => {
                    msg.code = String::from_utf8(field.value).ok()?;
                }
                6 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.is_stale = val != 0;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DiagnosticsError {
    pub path: String,
    pub error: String,
}

impl DiagnosticsError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        chunks.push(encode_string_field(2, &self.error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DiagnosticsRejected {
    pub path: String,
    pub reason: String,
}

impl DiagnosticsRejected {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        chunks.push(encode_string_field(2, &self.reason));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.reason = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DiagnosticsFileNotFound {
    pub path: String,
}

impl DiagnosticsFileNotFound {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DiagnosticsPermissionDenied {
    pub path: String,
}

impl DiagnosticsPermissionDenied {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EditArgs {
    pub path: String,
    pub stream_content: Option<String>,
}

impl EditArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        if let Some(ref val) = self.stream_content {
            chunks.push(encode_string_field_always(6, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                6 => {
                    msg.stream_content = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EditResultResult {
    Success(EditSuccess),
    FileNotFound(EditFileNotFound),
    ReadPermissionDenied(EditReadPermissionDenied),
    WritePermissionDenied(EditWritePermissionDenied),
    Rejected(EditRejected),
    Error(EditError),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EditResult {
    pub result: Option<EditResultResult>,
}

impl EditResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                EditResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                EditResultResult::FileNotFound(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                EditResultResult::ReadPermissionDenied(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
                EditResultResult::WritePermissionDenied(ref inner) => {
                    chunks.push(encode_message_field(4, &inner.encode()));
                }
                EditResultResult::Rejected(ref inner) => {
                    chunks.push(encode_message_field(6, &inner.encode()));
                }
                EditResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(7, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = EditSuccess::decode(&field.value)?;
                    msg.result = Some(EditResultResult::Success(val));
                }
                2 => {
                    let val = EditFileNotFound::decode(&field.value)?;
                    msg.result = Some(EditResultResult::FileNotFound(val));
                }
                3 => {
                    let val = EditReadPermissionDenied::decode(&field.value)?;
                    msg.result = Some(EditResultResult::ReadPermissionDenied(val));
                }
                4 => {
                    let val = EditWritePermissionDenied::decode(&field.value)?;
                    msg.result = Some(EditResultResult::WritePermissionDenied(val));
                }
                6 => {
                    let val = EditRejected::decode(&field.value)?;
                    msg.result = Some(EditResultResult::Rejected(val));
                }
                7 => {
                    let val = EditError::decode(&field.value)?;
                    msg.result = Some(EditResultResult::Error(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EditSuccess {
    pub path: String,
    pub lines_added: Option<i32>,
    pub lines_removed: Option<i32>,
    pub diff_string: Option<String>,
    pub before_full_file_content: Option<String>,
    pub after_full_file_content: String,
    pub message: Option<String>,
}

impl EditSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        if let Some(ref val) = self.lines_added {
            chunks.push(encode_varint_field_always(3, *val as u64));
        }
        if let Some(ref val) = self.lines_removed {
            chunks.push(encode_varint_field_always(4, *val as u64));
        }
        if let Some(ref val) = self.diff_string {
            chunks.push(encode_string_field_always(5, val));
        }
        if let Some(ref val) = self.before_full_file_content {
            chunks.push(encode_string_field_always(6, val));
        }
        chunks.push(encode_string_field(7, &self.after_full_file_content));
        if let Some(ref val) = self.message {
            chunks.push(encode_string_field_always(8, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.lines_added = Some(val as i32);
                }
                4 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.lines_removed = Some(val as i32);
                }
                5 => {
                    msg.diff_string = Some(String::from_utf8(field.value).ok()?);
                }
                6 => {
                    msg.before_full_file_content = Some(String::from_utf8(field.value).ok()?);
                }
                7 => {
                    msg.after_full_file_content = String::from_utf8(field.value).ok()?;
                }
                8 => {
                    msg.message = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EditFileNotFound {
    pub path: String,
}

impl EditFileNotFound {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EditReadPermissionDenied {
    pub path: String,
}

impl EditReadPermissionDenied {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EditWritePermissionDenied {
    pub path: String,
    pub error: String,
    pub is_readonly: bool,
}

impl EditWritePermissionDenied {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        chunks.push(encode_string_field(2, &self.error));
        chunks.push(encode_bool_field(3, self.is_readonly));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.is_readonly = val != 0;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EditRejected {
    pub path: String,
    pub reason: String,
}

impl EditRejected {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        chunks.push(encode_string_field(2, &self.reason));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.reason = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EditError {
    pub path: String,
    pub error: String,
    pub model_visible_error: Option<String>,
}

impl EditError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        chunks.push(encode_string_field(2, &self.error));
        if let Some(ref val) = self.model_visible_error {
            chunks.push(encode_string_field_always(5, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                5 => {
                    msg.model_visible_error = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EditToolCall {
    pub args: Option<EditArgs>,
    pub result: Option<EditResult>,
}

impl EditToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(EditArgs::decode(&field.value)?);
                }
                2 => {
                    msg.result = Some(EditResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EditToolCallDelta {
    pub stream_content_delta: String,
}

impl EditToolCallDelta {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.stream_content_delta));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.stream_content_delta = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExaFetchArgs {
    pub ids: Vec<String>,
    pub tool_call_id: String,
}

impl ExaFetchArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_repeated_string_field(1, &self.ids));
        chunks.push(encode_string_field(2, &self.tool_call_id));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.ids.push(String::from_utf8(field.value).ok()?);
                }
                2 => {
                    msg.tool_call_id = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExaFetchResultResult {
    Success(ExaFetchSuccess),
    Error(ExaFetchError),
    Rejected(ExaFetchRejected),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExaFetchResult {
    pub result: Option<ExaFetchResultResult>,
}

impl ExaFetchResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                ExaFetchResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                ExaFetchResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                ExaFetchResultResult::Rejected(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = ExaFetchSuccess::decode(&field.value)?;
                    msg.result = Some(ExaFetchResultResult::Success(val));
                }
                2 => {
                    let val = ExaFetchError::decode(&field.value)?;
                    msg.result = Some(ExaFetchResultResult::Error(val));
                }
                3 => {
                    let val = ExaFetchRejected::decode(&field.value)?;
                    msg.result = Some(ExaFetchResultResult::Rejected(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExaFetchSuccess {
    pub contents: Vec<ExaFetchContent>,
}

impl ExaFetchSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        let items_contents: Vec<Vec<u8>> = self.contents.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(1, &items_contents));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.contents.push(ExaFetchContent::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExaFetchError {
    pub error: String,
}

impl ExaFetchError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExaFetchRejected {
    pub reason: String,
}

impl ExaFetchRejected {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.reason));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.reason = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExaFetchContent {
    pub title: String,
    pub url: String,
    pub text: String,
    pub published_date: String,
}

impl ExaFetchContent {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.title));
        chunks.push(encode_string_field(2, &self.url));
        chunks.push(encode_string_field(3, &self.text));
        chunks.push(encode_string_field(4, &self.published_date));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.title = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.url = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.text = String::from_utf8(field.value).ok()?;
                }
                4 => {
                    msg.published_date = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExaFetchToolCall {
    pub args: Option<ExaFetchArgs>,
    pub result: Option<ExaFetchResult>,
}

impl ExaFetchToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(ExaFetchArgs::decode(&field.value)?);
                }
                2 => {
                    msg.result = Some(ExaFetchResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExaFetchRequestQuery {
    pub args: Option<ExaFetchArgs>,
}

impl ExaFetchRequestQuery {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(ExaFetchArgs::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExaFetchRequestResponseResult {
    Approved(ExaFetchRequestResponse_Approved),
    Rejected(ExaFetchRequestResponse_Rejected),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExaFetchRequestResponse {
    pub result: Option<ExaFetchRequestResponseResult>,
}

impl ExaFetchRequestResponse {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                ExaFetchRequestResponseResult::Approved(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                ExaFetchRequestResponseResult::Rejected(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = ExaFetchRequestResponse_Approved::decode(&field.value)?;
                    msg.result = Some(ExaFetchRequestResponseResult::Approved(val));
                }
                2 => {
                    let val = ExaFetchRequestResponse_Rejected::decode(&field.value)?;
                    msg.result = Some(ExaFetchRequestResponseResult::Rejected(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExaFetchRequestResponse_Approved {}

impl ExaFetchRequestResponse_Approved {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExaFetchRequestResponse_Rejected {
    pub reason: String,
}

impl ExaFetchRequestResponse_Rejected {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.reason));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.reason = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExaSearchArgs {
    pub query: String,
    pub type_: String,
    pub num_results: i32,
    pub tool_call_id: String,
}

impl ExaSearchArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.query));
        chunks.push(encode_string_field(2, &self.type_));
        if self.num_results != 0 {
            chunks.push(encode_varint_field_always(3, self.num_results as u64));
        }
        chunks.push(encode_string_field(4, &self.tool_call_id));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.query = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.type_ = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.num_results = val as i32;
                }
                4 => {
                    msg.tool_call_id = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExaSearchResultResult {
    Success(ExaSearchSuccess),
    Error(ExaSearchError),
    Rejected(ExaSearchRejected),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExaSearchResult {
    pub result: Option<ExaSearchResultResult>,
}

impl ExaSearchResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                ExaSearchResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                ExaSearchResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                ExaSearchResultResult::Rejected(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = ExaSearchSuccess::decode(&field.value)?;
                    msg.result = Some(ExaSearchResultResult::Success(val));
                }
                2 => {
                    let val = ExaSearchError::decode(&field.value)?;
                    msg.result = Some(ExaSearchResultResult::Error(val));
                }
                3 => {
                    let val = ExaSearchRejected::decode(&field.value)?;
                    msg.result = Some(ExaSearchResultResult::Rejected(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExaSearchSuccess {
    pub references: Vec<ExaSearchReference>,
}

impl ExaSearchSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        let items_references: Vec<Vec<u8>> =
            self.references.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(1, &items_references));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.references
                        .push(ExaSearchReference::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExaSearchError {
    pub error: String,
}

impl ExaSearchError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExaSearchRejected {
    pub reason: String,
}

impl ExaSearchRejected {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.reason));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.reason = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExaSearchReference {
    pub title: String,
    pub url: String,
    pub text: String,
    pub published_date: String,
}

impl ExaSearchReference {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.title));
        chunks.push(encode_string_field(2, &self.url));
        chunks.push(encode_string_field(3, &self.text));
        chunks.push(encode_string_field(4, &self.published_date));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.title = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.url = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.text = String::from_utf8(field.value).ok()?;
                }
                4 => {
                    msg.published_date = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExaSearchToolCall {
    pub args: Option<ExaSearchArgs>,
    pub result: Option<ExaSearchResult>,
}

impl ExaSearchToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(ExaSearchArgs::decode(&field.value)?);
                }
                2 => {
                    msg.result = Some(ExaSearchResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExaSearchRequestQuery {
    pub args: Option<ExaSearchArgs>,
}

impl ExaSearchRequestQuery {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(ExaSearchArgs::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExaSearchRequestResponseResult {
    Approved(ExaSearchRequestResponse_Approved),
    Rejected(ExaSearchRequestResponse_Rejected),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExaSearchRequestResponse {
    pub result: Option<ExaSearchRequestResponseResult>,
}

impl ExaSearchRequestResponse {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                ExaSearchRequestResponseResult::Approved(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                ExaSearchRequestResponseResult::Rejected(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = ExaSearchRequestResponse_Approved::decode(&field.value)?;
                    msg.result = Some(ExaSearchRequestResponseResult::Approved(val));
                }
                2 => {
                    let val = ExaSearchRequestResponse_Rejected::decode(&field.value)?;
                    msg.result = Some(ExaSearchRequestResponseResult::Rejected(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExaSearchRequestResponse_Approved {}

impl ExaSearchRequestResponse_Approved {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExaSearchRequestResponse_Rejected {
    pub reason: String,
}

impl ExaSearchRequestResponse_Rejected {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.reason));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.reason = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExecClientStreamClose {
    pub id: u32,
}

impl ExecClientStreamClose {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.id != 0 {
            chunks.push(encode_varint_field_always(1, self.id as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.id = val as u32;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExecClientThrow {
    pub id: u32,
    pub error: String,
    pub stack_trace: Option<String>,
}

impl ExecClientThrow {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.id != 0 {
            chunks.push(encode_varint_field_always(1, self.id as u64));
        }
        chunks.push(encode_string_field(2, &self.error));
        if let Some(ref val) = self.stack_trace {
            chunks.push(encode_string_field_always(3, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.id = val as u32;
                }
                2 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.stack_trace = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExecClientHeartbeat {
    pub id: u32,
}

impl ExecClientHeartbeat {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.id != 0 {
            chunks.push(encode_varint_field_always(1, self.id as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.id = val as u32;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecClientControlMessageMessage {
    StreamClose(ExecClientStreamClose),
    Throw(ExecClientThrow),
    Heartbeat(ExecClientHeartbeat),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExecClientControlMessage {
    pub message: Option<ExecClientControlMessageMessage>,
}

impl ExecClientControlMessage {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.message {
            match val {
                ExecClientControlMessageMessage::StreamClose(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                ExecClientControlMessageMessage::Throw(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                ExecClientControlMessageMessage::Heartbeat(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = ExecClientStreamClose::decode(&field.value)?;
                    msg.message = Some(ExecClientControlMessageMessage::StreamClose(val));
                }
                2 => {
                    let val = ExecClientThrow::decode(&field.value)?;
                    msg.message = Some(ExecClientControlMessageMessage::Throw(val));
                }
                3 => {
                    let val = ExecClientHeartbeat::decode(&field.value)?;
                    msg.message = Some(ExecClientControlMessageMessage::Heartbeat(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SpanContext {
    pub trace_id: String,
    pub span_id: String,
    pub trace_flags: Option<u32>,
    pub trace_state: Option<String>,
}

impl SpanContext {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.trace_id));
        chunks.push(encode_string_field(2, &self.span_id));
        if let Some(ref val) = self.trace_flags {
            chunks.push(encode_varint_field_always(3, *val as u64));
        }
        if let Some(ref val) = self.trace_state {
            chunks.push(encode_string_field_always(4, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.trace_id = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.span_id = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.trace_flags = Some(val as u32);
                }
                4 => {
                    msg.trace_state = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AbortArgs {}

impl AbortArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AbortResult {}

impl AbortResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecServerMessageMessage {
    ShellArgs(ShellArgs),
    WriteArgs(WriteArgs),
    DeleteArgs(DeleteArgs),
    GrepArgs(GrepArgs),
    ReadArgs(ReadArgs),
    LsArgs(LsArgs),
    DiagnosticsArgs(DiagnosticsArgs),
    RequestContextArgs(RequestContextArgs),
    McpArgs(McpArgs),
    ShellStreamArgs(ShellArgs),
    BackgroundShellSpawnArgs(BackgroundShellSpawnArgs),
    ListMcpResourcesExecArgs(ListMcpResourcesExecArgs),
    ReadMcpResourceExecArgs(ReadMcpResourceExecArgs),
    FetchArgs(FetchArgs),
    RecordScreenArgs(RecordScreenArgs),
    ComputerUseArgs(ComputerUseArgs),
    WriteShellStdinArgs(WriteShellStdinArgs),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExecServerMessage {
    pub id: u32,
    pub exec_id: String,
    pub span_context: Option<SpanContext>,
    pub message: Option<ExecServerMessageMessage>,
}

impl ExecServerMessage {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.id != 0 {
            chunks.push(encode_varint_field_always(1, self.id as u64));
        }
        chunks.push(encode_string_field(15, &self.exec_id));
        if let Some(ref val) = self.span_context {
            chunks.push(encode_message_field(19, &val.encode()));
        }
        if let Some(ref val) = self.message {
            match val {
                ExecServerMessageMessage::ShellArgs(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                ExecServerMessageMessage::WriteArgs(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
                ExecServerMessageMessage::DeleteArgs(ref inner) => {
                    chunks.push(encode_message_field(4, &inner.encode()));
                }
                ExecServerMessageMessage::GrepArgs(ref inner) => {
                    chunks.push(encode_message_field(5, &inner.encode()));
                }
                ExecServerMessageMessage::ReadArgs(ref inner) => {
                    chunks.push(encode_message_field(7, &inner.encode()));
                }
                ExecServerMessageMessage::LsArgs(ref inner) => {
                    chunks.push(encode_message_field(8, &inner.encode()));
                }
                ExecServerMessageMessage::DiagnosticsArgs(ref inner) => {
                    chunks.push(encode_message_field(9, &inner.encode()));
                }
                ExecServerMessageMessage::RequestContextArgs(ref inner) => {
                    chunks.push(encode_message_field(10, &inner.encode()));
                }
                ExecServerMessageMessage::McpArgs(ref inner) => {
                    chunks.push(encode_message_field(11, &inner.encode()));
                }
                ExecServerMessageMessage::ShellStreamArgs(ref inner) => {
                    chunks.push(encode_message_field(14, &inner.encode()));
                }
                ExecServerMessageMessage::BackgroundShellSpawnArgs(ref inner) => {
                    chunks.push(encode_message_field(16, &inner.encode()));
                }
                ExecServerMessageMessage::ListMcpResourcesExecArgs(ref inner) => {
                    chunks.push(encode_message_field(17, &inner.encode()));
                }
                ExecServerMessageMessage::ReadMcpResourceExecArgs(ref inner) => {
                    chunks.push(encode_message_field(18, &inner.encode()));
                }
                ExecServerMessageMessage::FetchArgs(ref inner) => {
                    chunks.push(encode_message_field(20, &inner.encode()));
                }
                ExecServerMessageMessage::RecordScreenArgs(ref inner) => {
                    chunks.push(encode_message_field(21, &inner.encode()));
                }
                ExecServerMessageMessage::ComputerUseArgs(ref inner) => {
                    chunks.push(encode_message_field(22, &inner.encode()));
                }
                ExecServerMessageMessage::WriteShellStdinArgs(ref inner) => {
                    chunks.push(encode_message_field(23, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.id = val as u32;
                }
                15 => {
                    msg.exec_id = String::from_utf8(field.value).ok()?;
                }
                19 => {
                    msg.span_context = Some(SpanContext::decode(&field.value)?);
                }
                2 => {
                    let val = ShellArgs::decode(&field.value)?;
                    msg.message = Some(ExecServerMessageMessage::ShellArgs(val));
                }
                3 => {
                    let val = WriteArgs::decode(&field.value)?;
                    msg.message = Some(ExecServerMessageMessage::WriteArgs(val));
                }
                4 => {
                    let val = DeleteArgs::decode(&field.value)?;
                    msg.message = Some(ExecServerMessageMessage::DeleteArgs(val));
                }
                5 => {
                    let val = GrepArgs::decode(&field.value)?;
                    msg.message = Some(ExecServerMessageMessage::GrepArgs(val));
                }
                7 => {
                    let val = ReadArgs::decode(&field.value)?;
                    msg.message = Some(ExecServerMessageMessage::ReadArgs(val));
                }
                8 => {
                    let val = LsArgs::decode(&field.value)?;
                    msg.message = Some(ExecServerMessageMessage::LsArgs(val));
                }
                9 => {
                    let val = DiagnosticsArgs::decode(&field.value)?;
                    msg.message = Some(ExecServerMessageMessage::DiagnosticsArgs(val));
                }
                10 => {
                    let val = RequestContextArgs::decode(&field.value)?;
                    msg.message = Some(ExecServerMessageMessage::RequestContextArgs(val));
                }
                11 => {
                    let val = McpArgs::decode(&field.value)?;
                    msg.message = Some(ExecServerMessageMessage::McpArgs(val));
                }
                14 => {
                    let val = ShellArgs::decode(&field.value)?;
                    msg.message = Some(ExecServerMessageMessage::ShellStreamArgs(val));
                }
                16 => {
                    let val = BackgroundShellSpawnArgs::decode(&field.value)?;
                    msg.message = Some(ExecServerMessageMessage::BackgroundShellSpawnArgs(val));
                }
                17 => {
                    let val = ListMcpResourcesExecArgs::decode(&field.value)?;
                    msg.message = Some(ExecServerMessageMessage::ListMcpResourcesExecArgs(val));
                }
                18 => {
                    let val = ReadMcpResourceExecArgs::decode(&field.value)?;
                    msg.message = Some(ExecServerMessageMessage::ReadMcpResourceExecArgs(val));
                }
                20 => {
                    let val = FetchArgs::decode(&field.value)?;
                    msg.message = Some(ExecServerMessageMessage::FetchArgs(val));
                }
                21 => {
                    let val = RecordScreenArgs::decode(&field.value)?;
                    msg.message = Some(ExecServerMessageMessage::RecordScreenArgs(val));
                }
                22 => {
                    let val = ComputerUseArgs::decode(&field.value)?;
                    msg.message = Some(ExecServerMessageMessage::ComputerUseArgs(val));
                }
                23 => {
                    let val = WriteShellStdinArgs::decode(&field.value)?;
                    msg.message = Some(ExecServerMessageMessage::WriteShellStdinArgs(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecClientMessageMessage {
    ShellResult(ShellResult),
    WriteResult(WriteResult),
    DeleteResult(DeleteResult),
    GrepResult(GrepResult),
    ReadResult(ReadResult),
    LsResult(LsResult),
    DiagnosticsResult(DiagnosticsResult),
    RequestContextResult(RequestContextResult),
    McpResult(McpResult),
    ShellStream(ShellStream),
    BackgroundShellSpawnResult(BackgroundShellSpawnResult),
    ListMcpResourcesExecResult(ListMcpResourcesExecResult),
    ReadMcpResourceExecResult(ReadMcpResourceExecResult),
    FetchResult(FetchResult),
    RecordScreenResult(RecordScreenResult),
    ComputerUseResult(ComputerUseResult),
    WriteShellStdinResult(WriteShellStdinResult),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExecClientMessage {
    pub id: u32,
    pub exec_id: String,
    pub message: Option<ExecClientMessageMessage>,
}

impl ExecClientMessage {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.id != 0 {
            chunks.push(encode_varint_field_always(1, self.id as u64));
        }
        chunks.push(encode_string_field(15, &self.exec_id));
        if let Some(ref val) = self.message {
            match val {
                ExecClientMessageMessage::ShellResult(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                ExecClientMessageMessage::WriteResult(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
                ExecClientMessageMessage::DeleteResult(ref inner) => {
                    chunks.push(encode_message_field(4, &inner.encode()));
                }
                ExecClientMessageMessage::GrepResult(ref inner) => {
                    chunks.push(encode_message_field(5, &inner.encode()));
                }
                ExecClientMessageMessage::ReadResult(ref inner) => {
                    chunks.push(encode_message_field(7, &inner.encode()));
                }
                ExecClientMessageMessage::LsResult(ref inner) => {
                    chunks.push(encode_message_field(8, &inner.encode()));
                }
                ExecClientMessageMessage::DiagnosticsResult(ref inner) => {
                    chunks.push(encode_message_field(9, &inner.encode()));
                }
                ExecClientMessageMessage::RequestContextResult(ref inner) => {
                    chunks.push(encode_message_field(10, &inner.encode()));
                }
                ExecClientMessageMessage::McpResult(ref inner) => {
                    chunks.push(encode_message_field(11, &inner.encode()));
                }
                ExecClientMessageMessage::ShellStream(ref inner) => {
                    chunks.push(encode_message_field(14, &inner.encode()));
                }
                ExecClientMessageMessage::BackgroundShellSpawnResult(ref inner) => {
                    chunks.push(encode_message_field(16, &inner.encode()));
                }
                ExecClientMessageMessage::ListMcpResourcesExecResult(ref inner) => {
                    chunks.push(encode_message_field(17, &inner.encode()));
                }
                ExecClientMessageMessage::ReadMcpResourceExecResult(ref inner) => {
                    chunks.push(encode_message_field(18, &inner.encode()));
                }
                ExecClientMessageMessage::FetchResult(ref inner) => {
                    chunks.push(encode_message_field(20, &inner.encode()));
                }
                ExecClientMessageMessage::RecordScreenResult(ref inner) => {
                    chunks.push(encode_message_field(21, &inner.encode()));
                }
                ExecClientMessageMessage::ComputerUseResult(ref inner) => {
                    chunks.push(encode_message_field(22, &inner.encode()));
                }
                ExecClientMessageMessage::WriteShellStdinResult(ref inner) => {
                    chunks.push(encode_message_field(23, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.id = val as u32;
                }
                15 => {
                    msg.exec_id = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    let val = ShellResult::decode(&field.value)?;
                    msg.message = Some(ExecClientMessageMessage::ShellResult(val));
                }
                3 => {
                    let val = WriteResult::decode(&field.value)?;
                    msg.message = Some(ExecClientMessageMessage::WriteResult(val));
                }
                4 => {
                    let val = DeleteResult::decode(&field.value)?;
                    msg.message = Some(ExecClientMessageMessage::DeleteResult(val));
                }
                5 => {
                    let val = GrepResult::decode(&field.value)?;
                    msg.message = Some(ExecClientMessageMessage::GrepResult(val));
                }
                7 => {
                    let val = ReadResult::decode(&field.value)?;
                    msg.message = Some(ExecClientMessageMessage::ReadResult(val));
                }
                8 => {
                    let val = LsResult::decode(&field.value)?;
                    msg.message = Some(ExecClientMessageMessage::LsResult(val));
                }
                9 => {
                    let val = DiagnosticsResult::decode(&field.value)?;
                    msg.message = Some(ExecClientMessageMessage::DiagnosticsResult(val));
                }
                10 => {
                    let val = RequestContextResult::decode(&field.value)?;
                    msg.message = Some(ExecClientMessageMessage::RequestContextResult(val));
                }
                11 => {
                    let val = McpResult::decode(&field.value)?;
                    msg.message = Some(ExecClientMessageMessage::McpResult(val));
                }
                14 => {
                    let val = ShellStream::decode(&field.value)?;
                    msg.message = Some(ExecClientMessageMessage::ShellStream(val));
                }
                16 => {
                    let val = BackgroundShellSpawnResult::decode(&field.value)?;
                    msg.message = Some(ExecClientMessageMessage::BackgroundShellSpawnResult(val));
                }
                17 => {
                    let val = ListMcpResourcesExecResult::decode(&field.value)?;
                    msg.message = Some(ExecClientMessageMessage::ListMcpResourcesExecResult(val));
                }
                18 => {
                    let val = ReadMcpResourceExecResult::decode(&field.value)?;
                    msg.message = Some(ExecClientMessageMessage::ReadMcpResourceExecResult(val));
                }
                20 => {
                    let val = FetchResult::decode(&field.value)?;
                    msg.message = Some(ExecClientMessageMessage::FetchResult(val));
                }
                21 => {
                    let val = RecordScreenResult::decode(&field.value)?;
                    msg.message = Some(ExecClientMessageMessage::RecordScreenResult(val));
                }
                22 => {
                    let val = ComputerUseResult::decode(&field.value)?;
                    msg.message = Some(ExecClientMessageMessage::ComputerUseResult(val));
                }
                23 => {
                    let val = WriteShellStdinResult::decode(&field.value)?;
                    msg.message = Some(ExecClientMessageMessage::WriteShellStdinResult(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FetchArgs {
    pub url: String,
    pub tool_call_id: String,
}

impl FetchArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.url));
        chunks.push(encode_string_field(2, &self.tool_call_id));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.url = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.tool_call_id = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FetchResultResult {
    Success(FetchSuccess),
    Error(FetchError),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FetchResult {
    pub result: Option<FetchResultResult>,
}

impl FetchResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                FetchResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                FetchResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = FetchSuccess::decode(&field.value)?;
                    msg.result = Some(FetchResultResult::Success(val));
                }
                2 => {
                    let val = FetchError::decode(&field.value)?;
                    msg.result = Some(FetchResultResult::Error(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FetchSuccess {
    pub url: String,
    pub content: String,
    pub status_code: i32,
    pub content_type: String,
}

impl FetchSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.url));
        chunks.push(encode_string_field(2, &self.content));
        if self.status_code != 0 {
            chunks.push(encode_varint_field_always(3, self.status_code as u64));
        }
        chunks.push(encode_string_field(4, &self.content_type));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.url = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.content = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.status_code = val as i32;
                }
                4 => {
                    msg.content_type = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FetchError {
    pub url: String,
    pub error: String,
}

impl FetchError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.url));
        chunks.push(encode_string_field(2, &self.error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.url = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GenerateImageArgs {
    pub description: String,
    pub file_path: Option<String>,
    pub reference_image_paths: Vec<String>,
}

impl GenerateImageArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.description));
        if let Some(ref val) = self.file_path {
            chunks.push(encode_string_field_always(2, val));
        }
        chunks.push(encode_repeated_string_field(5, &self.reference_image_paths));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.description = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.file_path = Some(String::from_utf8(field.value).ok()?);
                }
                5 => {
                    msg.reference_image_paths
                        .push(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GenerateImageResultResult {
    Success(GenerateImageSuccess),
    Error(GenerateImageError),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GenerateImageResult {
    pub result: Option<GenerateImageResultResult>,
}

impl GenerateImageResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                GenerateImageResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                GenerateImageResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = GenerateImageSuccess::decode(&field.value)?;
                    msg.result = Some(GenerateImageResultResult::Success(val));
                }
                2 => {
                    let val = GenerateImageError::decode(&field.value)?;
                    msg.result = Some(GenerateImageResultResult::Error(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GenerateImageSuccess {
    pub file_path: String,
    pub image_data: String,
}

impl GenerateImageSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.file_path));
        chunks.push(encode_string_field(2, &self.image_data));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.file_path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.image_data = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GenerateImageError {
    pub error: String,
}

impl GenerateImageError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GenerateImageToolCall {
    pub args: Option<GenerateImageArgs>,
    pub result: Option<GenerateImageResult>,
}

impl GenerateImageToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(GenerateImageArgs::decode(&field.value)?);
                }
                2 => {
                    msg.result = Some(GenerateImageResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GrepArgs {
    pub pattern: String,
    pub path: Option<String>,
    pub glob: Option<String>,
    pub output_mode: Option<String>,
    pub context_before: Option<i32>,
    pub context_after: Option<i32>,
    pub context: Option<i32>,
    pub case_insensitive: Option<bool>,
    pub type_: Option<String>,
    pub head_limit: Option<i32>,
    pub multiline: Option<bool>,
    pub sort: Option<String>,
    pub sort_ascending: Option<bool>,
    pub tool_call_id: String,
    pub sandbox_policy: Option<SandboxPolicy>,
}

impl GrepArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.pattern));
        if let Some(ref val) = self.path {
            chunks.push(encode_string_field_always(2, val));
        }
        if let Some(ref val) = self.glob {
            chunks.push(encode_string_field_always(3, val));
        }
        if let Some(ref val) = self.output_mode {
            chunks.push(encode_string_field_always(4, val));
        }
        if let Some(ref val) = self.context_before {
            chunks.push(encode_varint_field_always(5, *val as u64));
        }
        if let Some(ref val) = self.context_after {
            chunks.push(encode_varint_field_always(6, *val as u64));
        }
        if let Some(ref val) = self.context {
            chunks.push(encode_varint_field_always(7, *val as u64));
        }
        if let Some(ref val) = self.case_insensitive {
            chunks.push(encode_bool_field_always(8, *val));
        }
        if let Some(ref val) = self.type_ {
            chunks.push(encode_string_field_always(9, val));
        }
        if let Some(ref val) = self.head_limit {
            chunks.push(encode_varint_field_always(10, *val as u64));
        }
        if let Some(ref val) = self.multiline {
            chunks.push(encode_bool_field_always(11, *val));
        }
        if let Some(ref val) = self.sort {
            chunks.push(encode_string_field_always(12, val));
        }
        if let Some(ref val) = self.sort_ascending {
            chunks.push(encode_bool_field_always(13, *val));
        }
        chunks.push(encode_string_field(14, &self.tool_call_id));
        if let Some(ref val) = self.sandbox_policy {
            chunks.push(encode_message_field(15, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.pattern = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.path = Some(String::from_utf8(field.value).ok()?);
                }
                3 => {
                    msg.glob = Some(String::from_utf8(field.value).ok()?);
                }
                4 => {
                    msg.output_mode = Some(String::from_utf8(field.value).ok()?);
                }
                5 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.context_before = Some(val as i32);
                }
                6 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.context_after = Some(val as i32);
                }
                7 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.context = Some(val as i32);
                }
                8 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.case_insensitive = Some(val != 0);
                }
                9 => {
                    msg.type_ = Some(String::from_utf8(field.value).ok()?);
                }
                10 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.head_limit = Some(val as i32);
                }
                11 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.multiline = Some(val != 0);
                }
                12 => {
                    msg.sort = Some(String::from_utf8(field.value).ok()?);
                }
                13 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.sort_ascending = Some(val != 0);
                }
                14 => {
                    msg.tool_call_id = String::from_utf8(field.value).ok()?;
                }
                15 => {
                    msg.sandbox_policy = Some(SandboxPolicy::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GrepResultResult {
    Success(GrepSuccess),
    Error(GrepError),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GrepResult {
    pub result: Option<GrepResultResult>,
}

impl GrepResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                GrepResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                GrepResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = GrepSuccess::decode(&field.value)?;
                    msg.result = Some(GrepResultResult::Success(val));
                }
                2 => {
                    let val = GrepError::decode(&field.value)?;
                    msg.result = Some(GrepResultResult::Error(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GrepError {
    pub error: String,
}

impl GrepError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GrepSuccess {
    pub pattern: String,
    pub path: String,
    pub output_mode: String,
    pub workspace_results: std::collections::HashMap<String, GrepUnionResult>,
    pub active_editor_result: Option<GrepUnionResult>,
}

impl GrepSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.pattern));
        chunks.push(encode_string_field(2, &self.path));
        chunks.push(encode_string_field(3, &self.output_mode));
        for (key, val) in &self.workspace_results {
            let mut entry_chunks = Vec::new();
            entry_chunks.push(encode_string_field(1, key));
            entry_chunks.push(encode_message_field(2, &val.encode()));
            chunks.push(encode_message_field(4, &concat_bytes(&entry_chunks)));
        }
        if let Some(ref val) = self.active_editor_result {
            chunks.push(encode_message_field(5, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.pattern = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.output_mode = String::from_utf8(field.value).ok()?;
                }
                4 => {
                    let mut entry_key = String::new();
                    let mut entry_value = <GrepUnionResult>::default();
                    for sub in parse_proto_fields(&field.value) {
                        match sub.number {
                            1 => entry_key = String::from_utf8(sub.value).ok()?,
                            2 => {
                                entry_value = GrepUnionResult::decode(&sub.value)?;
                            }
                            _ => {}
                        }
                    }
                    msg.workspace_results.insert(entry_key, entry_value);
                }
                5 => {
                    msg.active_editor_result = Some(GrepUnionResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GrepUnionResultResult {
    Count(GrepCountResult),
    Files(GrepFilesResult),
    Content(GrepContentResult),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GrepUnionResult {
    pub result: Option<GrepUnionResultResult>,
}

impl GrepUnionResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                GrepUnionResultResult::Count(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                GrepUnionResultResult::Files(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                GrepUnionResultResult::Content(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = GrepCountResult::decode(&field.value)?;
                    msg.result = Some(GrepUnionResultResult::Count(val));
                }
                2 => {
                    let val = GrepFilesResult::decode(&field.value)?;
                    msg.result = Some(GrepUnionResultResult::Files(val));
                }
                3 => {
                    let val = GrepContentResult::decode(&field.value)?;
                    msg.result = Some(GrepUnionResultResult::Content(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GrepCountResult {
    pub counts: Vec<GrepFileCount>,
    pub total_files: i32,
    pub total_matches: i32,
    pub client_truncated: bool,
    pub ripgrep_truncated: bool,
}

impl GrepCountResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        let items_counts: Vec<Vec<u8>> = self.counts.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(1, &items_counts));
        if self.total_files != 0 {
            chunks.push(encode_varint_field_always(2, self.total_files as u64));
        }
        if self.total_matches != 0 {
            chunks.push(encode_varint_field_always(3, self.total_matches as u64));
        }
        chunks.push(encode_bool_field(4, self.client_truncated));
        chunks.push(encode_bool_field(5, self.ripgrep_truncated));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.counts.push(GrepFileCount::decode(&field.value)?);
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.total_files = val as i32;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.total_matches = val as i32;
                }
                4 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.client_truncated = val != 0;
                }
                5 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.ripgrep_truncated = val != 0;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GrepFileCount {
    pub file: String,
    pub count: i32,
}

impl GrepFileCount {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.file));
        if self.count != 0 {
            chunks.push(encode_varint_field_always(2, self.count as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.file = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.count = val as i32;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GrepFilesResult {
    pub files: Vec<String>,
    pub total_files: i32,
    pub client_truncated: bool,
    pub ripgrep_truncated: bool,
}

impl GrepFilesResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_repeated_string_field(1, &self.files));
        if self.total_files != 0 {
            chunks.push(encode_varint_field_always(2, self.total_files as u64));
        }
        chunks.push(encode_bool_field(3, self.client_truncated));
        chunks.push(encode_bool_field(4, self.ripgrep_truncated));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.files.push(String::from_utf8(field.value).ok()?);
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.total_files = val as i32;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.client_truncated = val != 0;
                }
                4 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.ripgrep_truncated = val != 0;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GrepContentResult {
    pub matches: Vec<GrepFileMatch>,
    pub total_lines: i32,
    pub total_matched_lines: i32,
    pub client_truncated: bool,
    pub ripgrep_truncated: bool,
}

impl GrepContentResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        let items_matches: Vec<Vec<u8>> = self.matches.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(1, &items_matches));
        if self.total_lines != 0 {
            chunks.push(encode_varint_field_always(2, self.total_lines as u64));
        }
        if self.total_matched_lines != 0 {
            chunks.push(encode_varint_field_always(
                3,
                self.total_matched_lines as u64,
            ));
        }
        chunks.push(encode_bool_field(4, self.client_truncated));
        chunks.push(encode_bool_field(5, self.ripgrep_truncated));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.matches.push(GrepFileMatch::decode(&field.value)?);
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.total_lines = val as i32;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.total_matched_lines = val as i32;
                }
                4 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.client_truncated = val != 0;
                }
                5 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.ripgrep_truncated = val != 0;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GrepFileMatch {
    pub file: String,
    pub matches: Vec<GrepContentMatch>,
}

impl GrepFileMatch {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.file));
        let items_matches: Vec<Vec<u8>> = self.matches.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(2, &items_matches));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.file = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.matches.push(GrepContentMatch::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GrepContentMatch {
    pub line_number: i32,
    pub content: String,
    pub content_truncated: bool,
    pub is_context_line: bool,
}

impl GrepContentMatch {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.line_number != 0 {
            chunks.push(encode_varint_field_always(1, self.line_number as u64));
        }
        chunks.push(encode_string_field(2, &self.content));
        chunks.push(encode_bool_field(3, self.content_truncated));
        chunks.push(encode_bool_field(4, self.is_context_line));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.line_number = val as i32;
                }
                2 => {
                    msg.content = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.content_truncated = val != 0;
                }
                4 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.is_context_line = val != 0;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GrepStream {
    pub pattern: String,
}

impl GrepStream {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.pattern));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.pattern = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GrepToolCall {
    pub args: Option<GrepArgs>,
    pub result: Option<GrepResult>,
}

impl GrepToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(GrepArgs::decode(&field.value)?);
                }
                2 => {
                    msg.result = Some(GrepResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GetBlobArgs {
    pub blob_id: Vec<u8>,
}

impl GetBlobArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if !self.blob_id.is_empty() {
            chunks.push(encode_message_field(1, &self.blob_id));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.blob_id = field.value;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GetBlobResult {
    pub blob_data: Option<Vec<u8>>,
}

impl GetBlobResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.blob_data {
            chunks.push(encode_message_field(1, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.blob_data = Some(field.value);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SetBlobArgs {
    pub blob_id: Vec<u8>,
    pub blob_data: Vec<u8>,
}

impl SetBlobArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if !self.blob_id.is_empty() {
            chunks.push(encode_message_field(1, &self.blob_id));
        }
        if !self.blob_data.is_empty() {
            chunks.push(encode_message_field(2, &self.blob_data));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.blob_id = field.value;
                }
                2 => {
                    msg.blob_data = field.value;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SetBlobResult {
    pub error: Option<Error>,
}

impl SetBlobResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.error {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.error = Some(Error::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum KvServerMessageMessage {
    GetBlobArgs(GetBlobArgs),
    SetBlobArgs(SetBlobArgs),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct KvServerMessage {
    pub id: u32,
    pub span_context: Option<SpanContext>,
    pub message: Option<KvServerMessageMessage>,
}

impl KvServerMessage {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.id != 0 {
            chunks.push(encode_varint_field_always(1, self.id as u64));
        }
        if let Some(ref val) = self.span_context {
            chunks.push(encode_message_field(4, &val.encode()));
        }
        if let Some(ref val) = self.message {
            match val {
                KvServerMessageMessage::GetBlobArgs(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                KvServerMessageMessage::SetBlobArgs(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.id = val as u32;
                }
                4 => {
                    msg.span_context = Some(SpanContext::decode(&field.value)?);
                }
                2 => {
                    let val = GetBlobArgs::decode(&field.value)?;
                    msg.message = Some(KvServerMessageMessage::GetBlobArgs(val));
                }
                3 => {
                    let val = SetBlobArgs::decode(&field.value)?;
                    msg.message = Some(KvServerMessageMessage::SetBlobArgs(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum KvClientMessageMessage {
    GetBlobResult(GetBlobResult),
    SetBlobResult(SetBlobResult),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct KvClientMessage {
    pub id: u32,
    pub message: Option<KvClientMessageMessage>,
}

impl KvClientMessage {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.id != 0 {
            chunks.push(encode_varint_field_always(1, self.id as u64));
        }
        if let Some(ref val) = self.message {
            match val {
                KvClientMessageMessage::GetBlobResult(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                KvClientMessageMessage::SetBlobResult(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.id = val as u32;
                }
                2 => {
                    let val = GetBlobResult::decode(&field.value)?;
                    msg.message = Some(KvClientMessageMessage::GetBlobResult(val));
                }
                3 => {
                    let val = SetBlobResult::decode(&field.value)?;
                    msg.message = Some(KvClientMessageMessage::SetBlobResult(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LsArgs {
    pub path: String,
    pub ignore: Vec<String>,
    pub tool_call_id: String,
    pub sandbox_policy: Option<SandboxPolicy>,
    pub timeout_ms: Option<u32>,
}

impl LsArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        chunks.push(encode_repeated_string_field(2, &self.ignore));
        chunks.push(encode_string_field(3, &self.tool_call_id));
        if let Some(ref val) = self.sandbox_policy {
            chunks.push(encode_message_field(4, &val.encode()));
        }
        if let Some(ref val) = self.timeout_ms {
            chunks.push(encode_varint_field_always(5, *val as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.ignore.push(String::from_utf8(field.value).ok()?);
                }
                3 => {
                    msg.tool_call_id = String::from_utf8(field.value).ok()?;
                }
                4 => {
                    msg.sandbox_policy = Some(SandboxPolicy::decode(&field.value)?);
                }
                5 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.timeout_ms = Some(val as u32);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LsResultResult {
    Success(LsSuccess),
    Error(LsError),
    Rejected(LsRejected),
    Timeout(LsTimeout),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LsResult {
    pub result: Option<LsResultResult>,
}

impl LsResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                LsResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                LsResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                LsResultResult::Rejected(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
                LsResultResult::Timeout(ref inner) => {
                    chunks.push(encode_message_field(4, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = LsSuccess::decode(&field.value)?;
                    msg.result = Some(LsResultResult::Success(val));
                }
                2 => {
                    let val = LsError::decode(&field.value)?;
                    msg.result = Some(LsResultResult::Error(val));
                }
                3 => {
                    let val = LsRejected::decode(&field.value)?;
                    msg.result = Some(LsResultResult::Rejected(val));
                }
                4 => {
                    let val = LsTimeout::decode(&field.value)?;
                    msg.result = Some(LsResultResult::Timeout(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LsSuccess {
    pub directory_tree_root: Option<LsDirectoryTreeNode>,
}

impl LsSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.directory_tree_root {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.directory_tree_root = Some(LsDirectoryTreeNode::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LsDirectoryTreeNode {
    pub abs_path: String,
    pub children_dirs: Vec<LsDirectoryTreeNode>,
    pub children_files: Vec<LsDirectoryTreeNode_File>,
    pub children_were_processed: bool,
    pub full_subtree_extension_counts: std::collections::HashMap<String, i32>,
    pub num_files: i32,
}

impl LsDirectoryTreeNode {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.abs_path));
        let items_children_dirs: Vec<Vec<u8>> = self
            .children_dirs
            .iter()
            .map(|item| item.encode())
            .collect();
        chunks.push(encode_repeated_message_field(2, &items_children_dirs));
        let items_children_files: Vec<Vec<u8>> = self
            .children_files
            .iter()
            .map(|item| item.encode())
            .collect();
        chunks.push(encode_repeated_message_field(3, &items_children_files));
        chunks.push(encode_bool_field(4, self.children_were_processed));
        for (key, val) in &self.full_subtree_extension_counts {
            let mut entry_chunks = Vec::new();
            entry_chunks.push(encode_string_field(1, key));
            entry_chunks.push(encode_varint_field_always(2, *val as u64));
            chunks.push(encode_message_field(5, &concat_bytes(&entry_chunks)));
        }
        if self.num_files != 0 {
            chunks.push(encode_varint_field_always(6, self.num_files as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.abs_path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.children_dirs
                        .push(LsDirectoryTreeNode::decode(&field.value)?);
                }
                3 => {
                    msg.children_files
                        .push(LsDirectoryTreeNode_File::decode(&field.value)?);
                }
                4 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.children_were_processed = val != 0;
                }
                5 => {
                    let mut entry_key = String::new();
                    let mut entry_value = <i32>::default();
                    for sub in parse_proto_fields(&field.value) {
                        match sub.number {
                            1 => entry_key = String::from_utf8(sub.value).ok()?,
                            2 => {
                                let (val, _) = decode_varint(&sub.value, 0)?;
                                entry_value = val as i32;
                            }
                            _ => {}
                        }
                    }
                    msg.full_subtree_extension_counts
                        .insert(entry_key, entry_value);
                }
                6 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.num_files = val as i32;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LsDirectoryTreeNode_File {
    pub name: String,
    pub terminal_metadata: Option<TerminalMetadata>,
}

impl LsDirectoryTreeNode_File {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.name));
        if let Some(ref val) = self.terminal_metadata {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.name = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.terminal_metadata = Some(TerminalMetadata::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LsError {
    pub path: String,
    pub error: String,
}

impl LsError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        chunks.push(encode_string_field(2, &self.error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LsRejected {
    pub path: String,
    pub reason: String,
}

impl LsRejected {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        chunks.push(encode_string_field(2, &self.reason));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.reason = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LsTimeout {
    pub directory_tree_root: Option<LsDirectoryTreeNode>,
}

impl LsTimeout {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.directory_tree_root {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.directory_tree_root = Some(LsDirectoryTreeNode::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TerminalMetadata {
    pub cwd: Option<String>,
    pub last_commands: Vec<TerminalMetadata_Command>,
    pub last_modified_ms: Option<i64>,
    pub current_command: Option<TerminalMetadata_Command>,
}

impl TerminalMetadata {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.cwd {
            chunks.push(encode_string_field_always(1, val));
        }
        let items_last_commands: Vec<Vec<u8>> = self
            .last_commands
            .iter()
            .map(|item| item.encode())
            .collect();
        chunks.push(encode_repeated_message_field(2, &items_last_commands));
        if let Some(ref val) = self.last_modified_ms {
            chunks.push(encode_varint_field_always(3, *val as u64));
        }
        if let Some(ref val) = self.current_command {
            chunks.push(encode_message_field(4, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.cwd = Some(String::from_utf8(field.value).ok()?);
                }
                2 => {
                    msg.last_commands
                        .push(TerminalMetadata_Command::decode(&field.value)?);
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.last_modified_ms = Some(val as i64);
                }
                4 => {
                    msg.current_command = Some(TerminalMetadata_Command::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TerminalMetadata_Command {
    pub command: String,
    pub exit_code: Option<i32>,
    pub timestamp_ms: Option<i64>,
    pub duration_ms: Option<i64>,
}

impl TerminalMetadata_Command {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.command));
        if let Some(ref val) = self.exit_code {
            chunks.push(encode_varint_field_always(2, *val as u64));
        }
        if let Some(ref val) = self.timestamp_ms {
            chunks.push(encode_varint_field_always(3, *val as u64));
        }
        if let Some(ref val) = self.duration_ms {
            chunks.push(encode_varint_field_always(4, *val as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.command = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.exit_code = Some(val as i32);
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.timestamp_ms = Some(val as i64);
                }
                4 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.duration_ms = Some(val as i64);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LsToolCall {
    pub args: Option<LsArgs>,
    pub result: Option<LsResult>,
}

impl LsToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(LsArgs::decode(&field.value)?);
                }
                2 => {
                    msg.result = Some(LsResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct McpArgs {
    pub name: String,
    pub args: std::collections::HashMap<String, Vec<u8>>,
    pub tool_call_id: String,
    pub provider_identifier: String,
    pub tool_name: String,
}

impl McpArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.name));
        for (key, val) in &self.args {
            let mut entry_chunks = Vec::new();
            entry_chunks.push(encode_string_field(1, key));
            entry_chunks.push(encode_message_field(2, val));
            chunks.push(encode_message_field(2, &concat_bytes(&entry_chunks)));
        }
        chunks.push(encode_string_field(3, &self.tool_call_id));
        chunks.push(encode_string_field(4, &self.provider_identifier));
        chunks.push(encode_string_field(5, &self.tool_name));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.name = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    let mut entry_key = String::new();
                    let mut entry_value = <Vec<u8>>::default();
                    for sub in parse_proto_fields(&field.value) {
                        match sub.number {
                            1 => entry_key = String::from_utf8(sub.value).ok()?,
                            2 => {
                                entry_value = sub.value;
                            }
                            _ => {}
                        }
                    }
                    msg.args.insert(entry_key, entry_value);
                }
                3 => {
                    msg.tool_call_id = String::from_utf8(field.value).ok()?;
                }
                4 => {
                    msg.provider_identifier = String::from_utf8(field.value).ok()?;
                }
                5 => {
                    msg.tool_name = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum McpResultResult {
    Success(McpSuccess),
    Error(McpError),
    Rejected(McpRejected),
    PermissionDenied(McpPermissionDenied),
    ToolNotFound(McpToolNotFound),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct McpResult {
    pub result: Option<McpResultResult>,
}

impl McpResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                McpResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                McpResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                McpResultResult::Rejected(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
                McpResultResult::PermissionDenied(ref inner) => {
                    chunks.push(encode_message_field(4, &inner.encode()));
                }
                McpResultResult::ToolNotFound(ref inner) => {
                    chunks.push(encode_message_field(5, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = McpSuccess::decode(&field.value)?;
                    msg.result = Some(McpResultResult::Success(val));
                }
                2 => {
                    let val = McpError::decode(&field.value)?;
                    msg.result = Some(McpResultResult::Error(val));
                }
                3 => {
                    let val = McpRejected::decode(&field.value)?;
                    msg.result = Some(McpResultResult::Rejected(val));
                }
                4 => {
                    let val = McpPermissionDenied::decode(&field.value)?;
                    msg.result = Some(McpResultResult::PermissionDenied(val));
                }
                5 => {
                    let val = McpToolNotFound::decode(&field.value)?;
                    msg.result = Some(McpResultResult::ToolNotFound(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct McpToolNotFound {
    pub name: String,
    pub available_tools: Vec<String>,
}

impl McpToolNotFound {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.name));
        chunks.push(encode_repeated_string_field(2, &self.available_tools));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.name = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.available_tools
                        .push(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct McpTextContent {
    pub text: String,
    pub output_location: Option<OutputLocation>,
}

impl McpTextContent {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.text));
        if let Some(ref val) = self.output_location {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.text = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.output_location = Some(OutputLocation::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct McpImageContent {
    pub data: Vec<u8>,
    pub mime_type: String,
}

impl McpImageContent {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if !self.data.is_empty() {
            chunks.push(encode_message_field(1, &self.data));
        }
        chunks.push(encode_string_field(2, &self.mime_type));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.data = field.value;
                }
                2 => {
                    msg.mime_type = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum McpToolResultContentItemContent {
    Text(McpTextContent),
    Image(McpImageContent),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct McpToolResultContentItem {
    pub content: Option<McpToolResultContentItemContent>,
}

impl McpToolResultContentItem {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.content {
            match val {
                McpToolResultContentItemContent::Text(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                McpToolResultContentItemContent::Image(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = McpTextContent::decode(&field.value)?;
                    msg.content = Some(McpToolResultContentItemContent::Text(val));
                }
                2 => {
                    let val = McpImageContent::decode(&field.value)?;
                    msg.content = Some(McpToolResultContentItemContent::Image(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct McpSuccess {
    pub content: Vec<McpToolResultContentItem>,
    pub is_error: bool,
}

impl McpSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        let items_content: Vec<Vec<u8>> = self.content.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(1, &items_content));
        chunks.push(encode_bool_field(2, self.is_error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.content
                        .push(McpToolResultContentItem::decode(&field.value)?);
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.is_error = val != 0;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct McpError {
    pub error: String,
}

impl McpError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct McpRejected {
    pub reason: String,
    pub is_readonly: bool,
}

impl McpRejected {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.reason));
        chunks.push(encode_bool_field(2, self.is_readonly));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.reason = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.is_readonly = val != 0;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct McpPermissionDenied {
    pub error: String,
    pub is_readonly: bool,
}

impl McpPermissionDenied {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.error));
        chunks.push(encode_bool_field(2, self.is_readonly));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.is_readonly = val != 0;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ListMcpResourcesExecArgs {
    pub server: Option<String>,
}

impl ListMcpResourcesExecArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.server {
            chunks.push(encode_string_field_always(1, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.server = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ListMcpResourcesExecResultResult {
    Success(ListMcpResourcesSuccess),
    Error(ListMcpResourcesError),
    Rejected(ListMcpResourcesRejected),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ListMcpResourcesExecResult {
    pub result: Option<ListMcpResourcesExecResultResult>,
}

impl ListMcpResourcesExecResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                ListMcpResourcesExecResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                ListMcpResourcesExecResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                ListMcpResourcesExecResultResult::Rejected(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = ListMcpResourcesSuccess::decode(&field.value)?;
                    msg.result = Some(ListMcpResourcesExecResultResult::Success(val));
                }
                2 => {
                    let val = ListMcpResourcesError::decode(&field.value)?;
                    msg.result = Some(ListMcpResourcesExecResultResult::Error(val));
                }
                3 => {
                    let val = ListMcpResourcesRejected::decode(&field.value)?;
                    msg.result = Some(ListMcpResourcesExecResultResult::Rejected(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ListMcpResourcesExecResult_McpResource {
    pub uri: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub mime_type: Option<String>,
    pub server: String,
    pub annotations: std::collections::HashMap<String, String>,
}

impl ListMcpResourcesExecResult_McpResource {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.uri));
        if let Some(ref val) = self.name {
            chunks.push(encode_string_field_always(2, val));
        }
        if let Some(ref val) = self.description {
            chunks.push(encode_string_field_always(3, val));
        }
        if let Some(ref val) = self.mime_type {
            chunks.push(encode_string_field_always(4, val));
        }
        chunks.push(encode_string_field(5, &self.server));
        for (key, val) in &self.annotations {
            let mut entry_chunks = Vec::new();
            entry_chunks.push(encode_string_field(1, key));
            entry_chunks.push(encode_string_field_always(2, val));
            chunks.push(encode_message_field(6, &concat_bytes(&entry_chunks)));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.uri = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.name = Some(String::from_utf8(field.value).ok()?);
                }
                3 => {
                    msg.description = Some(String::from_utf8(field.value).ok()?);
                }
                4 => {
                    msg.mime_type = Some(String::from_utf8(field.value).ok()?);
                }
                5 => {
                    msg.server = String::from_utf8(field.value).ok()?;
                }
                6 => {
                    let mut entry_key = String::new();
                    let mut entry_value = <String>::default();
                    for sub in parse_proto_fields(&field.value) {
                        match sub.number {
                            1 => entry_key = String::from_utf8(sub.value).ok()?,
                            2 => {
                                entry_value = String::from_utf8(sub.value).ok()?;
                            }
                            _ => {}
                        }
                    }
                    msg.annotations.insert(entry_key, entry_value);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ListMcpResourcesSuccess {
    pub resources: Vec<ListMcpResourcesExecResult_McpResource>,
}

impl ListMcpResourcesSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        let items_resources: Vec<Vec<u8>> =
            self.resources.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(1, &items_resources));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.resources
                        .push(ListMcpResourcesExecResult_McpResource::decode(
                            &field.value,
                        )?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ListMcpResourcesError {
    pub error: String,
}

impl ListMcpResourcesError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ListMcpResourcesRejected {
    pub reason: String,
}

impl ListMcpResourcesRejected {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.reason));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.reason = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadMcpResourceExecArgs {
    pub server: String,
    pub uri: String,
    pub download_path: Option<String>,
}

impl ReadMcpResourceExecArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.server));
        chunks.push(encode_string_field(2, &self.uri));
        if let Some(ref val) = self.download_path {
            chunks.push(encode_string_field_always(3, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.server = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.uri = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.download_path = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReadMcpResourceExecResultResult {
    Success(ReadMcpResourceSuccess),
    Error(ReadMcpResourceError),
    Rejected(ReadMcpResourceRejected),
    NotFound(ReadMcpResourceNotFound),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadMcpResourceExecResult {
    pub result: Option<ReadMcpResourceExecResultResult>,
}

impl ReadMcpResourceExecResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                ReadMcpResourceExecResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                ReadMcpResourceExecResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                ReadMcpResourceExecResultResult::Rejected(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
                ReadMcpResourceExecResultResult::NotFound(ref inner) => {
                    chunks.push(encode_message_field(4, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = ReadMcpResourceSuccess::decode(&field.value)?;
                    msg.result = Some(ReadMcpResourceExecResultResult::Success(val));
                }
                2 => {
                    let val = ReadMcpResourceError::decode(&field.value)?;
                    msg.result = Some(ReadMcpResourceExecResultResult::Error(val));
                }
                3 => {
                    let val = ReadMcpResourceRejected::decode(&field.value)?;
                    msg.result = Some(ReadMcpResourceExecResultResult::Rejected(val));
                }
                4 => {
                    let val = ReadMcpResourceNotFound::decode(&field.value)?;
                    msg.result = Some(ReadMcpResourceExecResultResult::NotFound(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReadMcpResourceSuccessContent {
    Text(String),
    Blob(Vec<u8>),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadMcpResourceSuccess {
    pub uri: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub mime_type: Option<String>,
    pub annotations: std::collections::HashMap<String, String>,
    pub download_path: Option<String>,
    pub content: Option<ReadMcpResourceSuccessContent>,
}

impl ReadMcpResourceSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.uri));
        if let Some(ref val) = self.name {
            chunks.push(encode_string_field_always(2, val));
        }
        if let Some(ref val) = self.description {
            chunks.push(encode_string_field_always(3, val));
        }
        if let Some(ref val) = self.mime_type {
            chunks.push(encode_string_field_always(4, val));
        }
        for (key, val) in &self.annotations {
            let mut entry_chunks = Vec::new();
            entry_chunks.push(encode_string_field(1, key));
            entry_chunks.push(encode_string_field_always(2, val));
            chunks.push(encode_message_field(7, &concat_bytes(&entry_chunks)));
        }
        if let Some(ref val) = self.download_path {
            chunks.push(encode_string_field_always(8, val));
        }
        if let Some(ref val) = self.content {
            match val {
                ReadMcpResourceSuccessContent::Text(ref inner) => {
                    chunks.push(encode_string_field_always(5, inner));
                }
                ReadMcpResourceSuccessContent::Blob(ref inner) => {
                    chunks.push(encode_message_field(6, inner));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.uri = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.name = Some(String::from_utf8(field.value).ok()?);
                }
                3 => {
                    msg.description = Some(String::from_utf8(field.value).ok()?);
                }
                4 => {
                    msg.mime_type = Some(String::from_utf8(field.value).ok()?);
                }
                7 => {
                    let mut entry_key = String::new();
                    let mut entry_value = <String>::default();
                    for sub in parse_proto_fields(&field.value) {
                        match sub.number {
                            1 => entry_key = String::from_utf8(sub.value).ok()?,
                            2 => {
                                entry_value = String::from_utf8(sub.value).ok()?;
                            }
                            _ => {}
                        }
                    }
                    msg.annotations.insert(entry_key, entry_value);
                }
                8 => {
                    msg.download_path = Some(String::from_utf8(field.value).ok()?);
                }
                5 => {
                    let val = String::from_utf8(field.value).ok()?;
                    msg.content = Some(ReadMcpResourceSuccessContent::Text(val));
                }
                6 => {
                    msg.content = Some(ReadMcpResourceSuccessContent::Blob(field.value));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadMcpResourceError {
    pub uri: String,
    pub error: String,
}

impl ReadMcpResourceError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.uri));
        chunks.push(encode_string_field(2, &self.error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.uri = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadMcpResourceRejected {
    pub uri: String,
    pub reason: String,
}

impl ReadMcpResourceRejected {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.uri));
        chunks.push(encode_string_field(2, &self.reason));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.uri = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.reason = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadMcpResourceNotFound {
    pub uri: String,
}

impl ReadMcpResourceNotFound {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.uri));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.uri = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct McpToolDefinition {
    pub name: String,
    pub provider_identifier: String,
    pub tool_name: String,
    pub description: String,
    pub input_schema: Vec<u8>,
}

impl McpToolDefinition {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.name));
        chunks.push(encode_string_field(4, &self.provider_identifier));
        chunks.push(encode_string_field(5, &self.tool_name));
        chunks.push(encode_string_field(2, &self.description));
        if !self.input_schema.is_empty() {
            chunks.push(encode_message_field(3, &self.input_schema));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.name = String::from_utf8(field.value).ok()?;
                }
                4 => {
                    msg.provider_identifier = String::from_utf8(field.value).ok()?;
                }
                5 => {
                    msg.tool_name = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.description = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.input_schema = field.value;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct McpTools {
    pub mcp_tools: Vec<McpToolDefinition>,
}

impl McpTools {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        let items_mcp_tools: Vec<Vec<u8>> =
            self.mcp_tools.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(1, &items_mcp_tools));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.mcp_tools.push(McpToolDefinition::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct McpInstructions {
    pub server_name: String,
    pub instructions: String,
}

impl McpInstructions {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.server_name));
        chunks.push(encode_string_field(2, &self.instructions));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.server_name = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.instructions = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct McpDescriptor {
    pub server_name: String,
    pub server_identifier: String,
    pub folder_path: Option<String>,
    pub server_use_instructions: Option<String>,
    pub tools: Vec<McpToolDescriptor>,
}

impl McpDescriptor {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.server_name));
        chunks.push(encode_string_field(2, &self.server_identifier));
        if let Some(ref val) = self.folder_path {
            chunks.push(encode_string_field_always(3, val));
        }
        if let Some(ref val) = self.server_use_instructions {
            chunks.push(encode_string_field_always(4, val));
        }
        let items_tools: Vec<Vec<u8>> = self.tools.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(5, &items_tools));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.server_name = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.server_identifier = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.folder_path = Some(String::from_utf8(field.value).ok()?);
                }
                4 => {
                    msg.server_use_instructions = Some(String::from_utf8(field.value).ok()?);
                }
                5 => {
                    msg.tools.push(McpToolDescriptor::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct McpToolDescriptor {
    pub tool_name: String,
    pub definition_path: Option<String>,
}

impl McpToolDescriptor {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.tool_name));
        if let Some(ref val) = self.definition_path {
            chunks.push(encode_string_field_always(2, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.tool_name = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.definition_path = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct McpFileSystemOptions {
    pub enabled: bool,
    pub workspace_project_dir: String,
    pub mcp_descriptors: Vec<McpDescriptor>,
}

impl McpFileSystemOptions {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_bool_field(1, self.enabled));
        chunks.push(encode_string_field(2, &self.workspace_project_dir));
        let items_mcp_descriptors: Vec<Vec<u8>> = self
            .mcp_descriptors
            .iter()
            .map(|item| item.encode())
            .collect();
        chunks.push(encode_repeated_message_field(3, &items_mcp_descriptors));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.enabled = val != 0;
                }
                2 => {
                    msg.workspace_project_dir = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.mcp_descriptors
                        .push(McpDescriptor::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadArgs {
    pub path: String,
    pub tool_call_id: String,
}

impl ReadArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        chunks.push(encode_string_field(2, &self.tool_call_id));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.tool_call_id = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReadResultResult {
    Success(ReadSuccess),
    Error(ReadError),
    Rejected(ReadRejected),
    FileNotFound(ReadFileNotFound),
    PermissionDenied(ReadPermissionDenied),
    InvalidFile(ReadInvalidFile),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadResult {
    pub result: Option<ReadResultResult>,
}

impl ReadResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                ReadResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                ReadResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                ReadResultResult::Rejected(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
                ReadResultResult::FileNotFound(ref inner) => {
                    chunks.push(encode_message_field(4, &inner.encode()));
                }
                ReadResultResult::PermissionDenied(ref inner) => {
                    chunks.push(encode_message_field(5, &inner.encode()));
                }
                ReadResultResult::InvalidFile(ref inner) => {
                    chunks.push(encode_message_field(6, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = ReadSuccess::decode(&field.value)?;
                    msg.result = Some(ReadResultResult::Success(val));
                }
                2 => {
                    let val = ReadError::decode(&field.value)?;
                    msg.result = Some(ReadResultResult::Error(val));
                }
                3 => {
                    let val = ReadRejected::decode(&field.value)?;
                    msg.result = Some(ReadResultResult::Rejected(val));
                }
                4 => {
                    let val = ReadFileNotFound::decode(&field.value)?;
                    msg.result = Some(ReadResultResult::FileNotFound(val));
                }
                5 => {
                    let val = ReadPermissionDenied::decode(&field.value)?;
                    msg.result = Some(ReadResultResult::PermissionDenied(val));
                }
                6 => {
                    let val = ReadInvalidFile::decode(&field.value)?;
                    msg.result = Some(ReadResultResult::InvalidFile(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReadSuccessOutput {
    Content(String),
    Data(Vec<u8>),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadSuccess {
    pub path: String,
    pub total_lines: i32,
    pub file_size: i64,
    pub truncated: bool,
    pub output_blob_id: Option<Vec<u8>>,
    pub output: Option<ReadSuccessOutput>,
}

impl ReadSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        if self.total_lines != 0 {
            chunks.push(encode_varint_field_always(3, self.total_lines as u64));
        }
        if self.file_size != 0 {
            chunks.push(encode_varint_field_always(4, self.file_size as u64));
        }
        chunks.push(encode_bool_field(6, self.truncated));
        if let Some(ref val) = self.output_blob_id {
            chunks.push(encode_message_field(7, val));
        }
        if let Some(ref val) = self.output {
            match val {
                ReadSuccessOutput::Content(ref inner) => {
                    chunks.push(encode_string_field_always(2, inner));
                }
                ReadSuccessOutput::Data(ref inner) => {
                    chunks.push(encode_message_field(5, inner));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.total_lines = val as i32;
                }
                4 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.file_size = val as i64;
                }
                6 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.truncated = val != 0;
                }
                7 => {
                    msg.output_blob_id = Some(field.value);
                }
                2 => {
                    let val = String::from_utf8(field.value).ok()?;
                    msg.output = Some(ReadSuccessOutput::Content(val));
                }
                5 => {
                    msg.output = Some(ReadSuccessOutput::Data(field.value));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadError {
    pub path: String,
    pub error: String,
}

impl ReadError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        chunks.push(encode_string_field(2, &self.error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadRejected {
    pub path: String,
    pub reason: String,
}

impl ReadRejected {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        chunks.push(encode_string_field(2, &self.reason));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.reason = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadFileNotFound {
    pub path: String,
}

impl ReadFileNotFound {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadPermissionDenied {
    pub path: String,
}

impl ReadPermissionDenied {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadInvalidFile {
    pub path: String,
    pub reason: String,
}

impl ReadInvalidFile {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        chunks.push(encode_string_field(2, &self.reason));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.reason = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadToolCall {
    pub args: Option<ReadToolArgs>,
    pub result: Option<ReadToolResult>,
}

impl ReadToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(ReadToolArgs::decode(&field.value)?);
                }
                2 => {
                    msg.result = Some(ReadToolResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadToolArgs {
    pub path: String,
    pub offset: Option<i32>,
    pub limit: Option<i32>,
}

impl ReadToolArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        if let Some(ref val) = self.offset {
            chunks.push(encode_varint_field_always(2, *val as u64));
        }
        if let Some(ref val) = self.limit {
            chunks.push(encode_varint_field_always(3, *val as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.offset = Some(val as i32);
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.limit = Some(val as i32);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReadToolResultResult {
    Success(ReadToolSuccess),
    Error(ReadToolError),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadToolResult {
    pub result: Option<ReadToolResultResult>,
}

impl ReadToolResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                ReadToolResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                ReadToolResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = ReadToolSuccess::decode(&field.value)?;
                    msg.result = Some(ReadToolResultResult::Success(val));
                }
                2 => {
                    let val = ReadToolError::decode(&field.value)?;
                    msg.result = Some(ReadToolResultResult::Error(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadRange {
    pub start_line: u32,
    pub end_line: u32,
}

impl ReadRange {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.start_line != 0 {
            chunks.push(encode_varint_field_always(1, self.start_line as u64));
        }
        if self.end_line != 0 {
            chunks.push(encode_varint_field_always(2, self.end_line as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.start_line = val as u32;
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.end_line = val as u32;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReadToolSuccessOutput {
    Content(String),
    Data(Vec<u8>),
    DataBlobId(Vec<u8>),
    ContentBlobId(Vec<u8>),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadToolSuccess {
    pub is_empty: bool,
    pub exceeded_limit: bool,
    pub total_lines: u32,
    pub file_size: u32,
    pub path: String,
    pub read_range: Option<ReadRange>,
    pub output: Option<ReadToolSuccessOutput>,
}

impl ReadToolSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_bool_field(2, self.is_empty));
        chunks.push(encode_bool_field(3, self.exceeded_limit));
        if self.total_lines != 0 {
            chunks.push(encode_varint_field_always(4, self.total_lines as u64));
        }
        if self.file_size != 0 {
            chunks.push(encode_varint_field_always(5, self.file_size as u64));
        }
        chunks.push(encode_string_field(7, &self.path));
        if let Some(ref val) = self.read_range {
            chunks.push(encode_message_field(8, &val.encode()));
        }
        if let Some(ref val) = self.output {
            match val {
                ReadToolSuccessOutput::Content(ref inner) => {
                    chunks.push(encode_string_field_always(1, inner));
                }
                ReadToolSuccessOutput::Data(ref inner) => {
                    chunks.push(encode_message_field(6, inner));
                }
                ReadToolSuccessOutput::DataBlobId(ref inner) => {
                    chunks.push(encode_message_field(9, inner));
                }
                ReadToolSuccessOutput::ContentBlobId(ref inner) => {
                    chunks.push(encode_message_field(10, inner));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.is_empty = val != 0;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.exceeded_limit = val != 0;
                }
                4 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.total_lines = val as u32;
                }
                5 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.file_size = val as u32;
                }
                7 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                8 => {
                    msg.read_range = Some(ReadRange::decode(&field.value)?);
                }
                1 => {
                    let val = String::from_utf8(field.value).ok()?;
                    msg.output = Some(ReadToolSuccessOutput::Content(val));
                }
                6 => {
                    msg.output = Some(ReadToolSuccessOutput::Data(field.value));
                }
                9 => {
                    msg.output = Some(ReadToolSuccessOutput::DataBlobId(field.value));
                }
                10 => {
                    msg.output = Some(ReadToolSuccessOutput::ContentBlobId(field.value));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadToolError {
    pub error_message: String,
}

impl ReadToolError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.error_message));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.error_message = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RecordScreenArgs {
    pub mode: i32,
    pub tool_call_id: String,
    pub save_as_filename: Option<String>,
}

impl RecordScreenArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.mode != 0 {
            chunks.push(encode_varint_field_always(1, self.mode as u64));
        }
        chunks.push(encode_string_field(2, &self.tool_call_id));
        if let Some(ref val) = self.save_as_filename {
            chunks.push(encode_string_field_always(3, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.mode = val as i32;
                }
                2 => {
                    msg.tool_call_id = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.save_as_filename = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecordScreenResultResult {
    StartSuccess(RecordScreenStartSuccess),
    SaveSuccess(RecordScreenSaveSuccess),
    DiscardSuccess(RecordScreenDiscardSuccess),
    Failure(RecordScreenFailure),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RecordScreenResult {
    pub result: Option<RecordScreenResultResult>,
}

impl RecordScreenResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                RecordScreenResultResult::StartSuccess(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                RecordScreenResultResult::SaveSuccess(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                RecordScreenResultResult::DiscardSuccess(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
                RecordScreenResultResult::Failure(ref inner) => {
                    chunks.push(encode_message_field(4, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = RecordScreenStartSuccess::decode(&field.value)?;
                    msg.result = Some(RecordScreenResultResult::StartSuccess(val));
                }
                2 => {
                    let val = RecordScreenSaveSuccess::decode(&field.value)?;
                    msg.result = Some(RecordScreenResultResult::SaveSuccess(val));
                }
                3 => {
                    let val = RecordScreenDiscardSuccess::decode(&field.value)?;
                    msg.result = Some(RecordScreenResultResult::DiscardSuccess(val));
                }
                4 => {
                    let val = RecordScreenFailure::decode(&field.value)?;
                    msg.result = Some(RecordScreenResultResult::Failure(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RecordScreenStartSuccess {
    pub was_prior_recording_cancelled: bool,
    pub was_save_as_filename_ignored: bool,
}

impl RecordScreenStartSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_bool_field(1, self.was_prior_recording_cancelled));
        chunks.push(encode_bool_field(2, self.was_save_as_filename_ignored));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.was_prior_recording_cancelled = val != 0;
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.was_save_as_filename_ignored = val != 0;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RecordScreenSaveSuccess {
    pub path: String,
    pub recording_duration_ms: i64,
    pub requested_file_path_rejected_reason: Option<i32>,
}

impl RecordScreenSaveSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        if self.recording_duration_ms != 0 {
            chunks.push(encode_varint_field_always(
                2,
                self.recording_duration_ms as u64,
            ));
        }
        if let Some(ref val) = self.requested_file_path_rejected_reason {
            chunks.push(encode_varint_field_always(3, *val as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.recording_duration_ms = val as i64;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.requested_file_path_rejected_reason = Some(val as i32);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RecordScreenDiscardSuccess {}

impl RecordScreenDiscardSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RecordScreenFailure {
    pub error: String,
}

impl RecordScreenFailure {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CursorPackagePrompt {
    pub name: String,
    pub file_path: String,
}

impl CursorPackagePrompt {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.name));
        chunks.push(encode_string_field(2, &self.file_path));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.name = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.file_path = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CursorPackage {
    pub name: String,
    pub description: String,
    pub folder_path: String,
    pub enabled: bool,
    pub parse_error: Option<String>,
    pub prompts: Vec<CursorPackagePrompt>,
    pub readme_file_path: String,
    pub package_type: i32,
}

impl CursorPackage {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.name));
        chunks.push(encode_string_field(2, &self.description));
        chunks.push(encode_string_field(3, &self.folder_path));
        chunks.push(encode_bool_field(4, self.enabled));
        if let Some(ref val) = self.parse_error {
            chunks.push(encode_string_field_always(5, val));
        }
        let items_prompts: Vec<Vec<u8>> = self.prompts.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(6, &items_prompts));
        chunks.push(encode_string_field(7, &self.readme_file_path));
        if self.package_type != 0 {
            chunks.push(encode_varint_field_always(8, self.package_type as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.name = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.description = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.folder_path = String::from_utf8(field.value).ok()?;
                }
                4 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.enabled = val != 0;
                }
                5 => {
                    msg.parse_error = Some(String::from_utf8(field.value).ok()?);
                }
                6 => {
                    msg.prompts.push(CursorPackagePrompt::decode(&field.value)?);
                }
                7 => {
                    msg.readme_file_path = String::from_utf8(field.value).ok()?;
                }
                8 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.package_type = val as i32;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RepositoryIndexingInfo {
    pub relative_workspace_path: String,
    pub remote_urls: Vec<String>,
    pub remote_names: Vec<String>,
    pub repo_name: String,
    pub repo_owner: String,
    pub is_tracked: bool,
    pub is_local: bool,
    pub orthogonal_transform_seed: Option<f64>,
    pub workspace_uri: String,
    pub path_encryption_key: String,
}

impl RepositoryIndexingInfo {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.relative_workspace_path));
        chunks.push(encode_repeated_string_field(2, &self.remote_urls));
        chunks.push(encode_repeated_string_field(3, &self.remote_names));
        chunks.push(encode_string_field(4, &self.repo_name));
        chunks.push(encode_string_field(5, &self.repo_owner));
        chunks.push(encode_bool_field(6, self.is_tracked));
        chunks.push(encode_bool_field(7, self.is_local));
        if let Some(ref val) = self.orthogonal_transform_seed {
            chunks.push(encode_double_field_always(8, *val));
        }
        chunks.push(encode_string_field(9, &self.workspace_uri));
        chunks.push(encode_string_field(10, &self.path_encryption_key));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.relative_workspace_path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.remote_urls.push(String::from_utf8(field.value).ok()?);
                }
                3 => {
                    msg.remote_names.push(String::from_utf8(field.value).ok()?);
                }
                4 => {
                    msg.repo_name = String::from_utf8(field.value).ok()?;
                }
                5 => {
                    msg.repo_owner = String::from_utf8(field.value).ok()?;
                }
                6 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.is_tracked = val != 0;
                }
                7 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.is_local = val != 0;
                }
                8 => {
                    let val = f64::from_le_bytes(field.value.try_into().ok()?);
                    msg.orthogonal_transform_seed = Some(val);
                }
                9 => {
                    msg.workspace_uri = String::from_utf8(field.value).ok()?;
                }
                10 => {
                    msg.path_encryption_key = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RequestContextArgs {
    pub notes_session_id: Option<String>,
    pub workspace_id: Option<String>,
}

impl RequestContextArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.notes_session_id {
            chunks.push(encode_string_field_always(2, val));
        }
        if let Some(ref val) = self.workspace_id {
            chunks.push(encode_string_field_always(3, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                2 => {
                    msg.notes_session_id = Some(String::from_utf8(field.value).ok()?);
                }
                3 => {
                    msg.workspace_id = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RequestContextResultResult {
    Success(RequestContextSuccess),
    Error(RequestContextError),
    Rejected(RequestContextRejected),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RequestContextResult {
    pub result: Option<RequestContextResultResult>,
}

impl RequestContextResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                RequestContextResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                RequestContextResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                RequestContextResultResult::Rejected(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = RequestContextSuccess::decode(&field.value)?;
                    msg.result = Some(RequestContextResultResult::Success(val));
                }
                2 => {
                    let val = RequestContextError::decode(&field.value)?;
                    msg.result = Some(RequestContextResultResult::Error(val));
                }
                3 => {
                    let val = RequestContextRejected::decode(&field.value)?;
                    msg.result = Some(RequestContextResultResult::Rejected(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RequestContextSuccess {
    pub request_context: Option<RequestContext>,
}

impl RequestContextSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.request_context {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.request_context = Some(RequestContext::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RequestContextError {
    pub error: String,
}

impl RequestContextError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RequestContextRejected {
    pub reason: String,
}

impl RequestContextRejected {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.reason));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.reason = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ImageProto {
    pub data: Vec<u8>,
    pub uuid: String,
    pub path: String,
    pub dimension: Option<ImageProto_Dimension>,
    pub task_specific_description: Option<String>,
    pub mime_type: String,
}

impl ImageProto {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if !self.data.is_empty() {
            chunks.push(encode_message_field(1, &self.data));
        }
        chunks.push(encode_string_field(2, &self.uuid));
        chunks.push(encode_string_field(3, &self.path));
        if let Some(ref val) = self.dimension {
            chunks.push(encode_message_field(4, &val.encode()));
        }
        if let Some(ref val) = self.task_specific_description {
            chunks.push(encode_string_field_always(6, val));
        }
        chunks.push(encode_string_field(7, &self.mime_type));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.data = field.value;
                }
                2 => {
                    msg.uuid = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                4 => {
                    msg.dimension = Some(ImageProto_Dimension::decode(&field.value)?);
                }
                6 => {
                    msg.task_specific_description = Some(String::from_utf8(field.value).ok()?);
                }
                7 => {
                    msg.mime_type = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ImageProto_Dimension {
    pub width: i32,
    pub height: i32,
}

impl ImageProto_Dimension {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.width != 0 {
            chunks.push(encode_varint_field_always(1, self.width as u64));
        }
        if self.height != 0 {
            chunks.push(encode_varint_field_always(2, self.height as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.width = val as i32;
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.height = val as i32;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GitRepoInfo {
    pub path: String,
    pub status: String,
    pub branch_name: String,
    pub remote_url: Option<String>,
}

impl GitRepoInfo {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        chunks.push(encode_string_field(2, &self.status));
        chunks.push(encode_string_field(3, &self.branch_name));
        if let Some(ref val) = self.remote_url {
            chunks.push(encode_string_field_always(4, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.status = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.branch_name = String::from_utf8(field.value).ok()?;
                }
                4 => {
                    msg.remote_url = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RequestContextEnv {
    pub os_version: String,
    pub workspace_paths: Vec<String>,
    pub shell: String,
    pub sandbox_enabled: bool,
    pub terminals_folder: String,
    pub agent_shared_notes_folder: String,
    pub agent_conversation_notes_folder: String,
    pub time_zone: String,
    pub project_folder: String,
    pub agent_transcripts_folder: String,
}

impl RequestContextEnv {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.os_version));
        chunks.push(encode_repeated_string_field(2, &self.workspace_paths));
        chunks.push(encode_string_field(3, &self.shell));
        chunks.push(encode_bool_field(5, self.sandbox_enabled));
        chunks.push(encode_string_field(7, &self.terminals_folder));
        chunks.push(encode_string_field(8, &self.agent_shared_notes_folder));
        chunks.push(encode_string_field(
            9,
            &self.agent_conversation_notes_folder,
        ));
        chunks.push(encode_string_field(10, &self.time_zone));
        chunks.push(encode_string_field(11, &self.project_folder));
        chunks.push(encode_string_field(12, &self.agent_transcripts_folder));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.os_version = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.workspace_paths
                        .push(String::from_utf8(field.value).ok()?);
                }
                3 => {
                    msg.shell = String::from_utf8(field.value).ok()?;
                }
                5 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.sandbox_enabled = val != 0;
                }
                7 => {
                    msg.terminals_folder = String::from_utf8(field.value).ok()?;
                }
                8 => {
                    msg.agent_shared_notes_folder = String::from_utf8(field.value).ok()?;
                }
                9 => {
                    msg.agent_conversation_notes_folder = String::from_utf8(field.value).ok()?;
                }
                10 => {
                    msg.time_zone = String::from_utf8(field.value).ok()?;
                }
                11 => {
                    msg.project_folder = String::from_utf8(field.value).ok()?;
                }
                12 => {
                    msg.agent_transcripts_folder = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DebugModeConfig {
    pub log_path: String,
    pub server_endpoint: String,
}

impl DebugModeConfig {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.log_path));
        chunks.push(encode_string_field(2, &self.server_endpoint));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.log_path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.server_endpoint = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SkillDescriptor {
    pub name: String,
    pub description: String,
    pub folder_path: String,
    pub enabled: bool,
    pub parse_error: Option<String>,
    pub readme_file_path: String,
    pub package_type: i32,
}

impl SkillDescriptor {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.name));
        chunks.push(encode_string_field(2, &self.description));
        chunks.push(encode_string_field(3, &self.folder_path));
        chunks.push(encode_bool_field(4, self.enabled));
        if let Some(ref val) = self.parse_error {
            chunks.push(encode_string_field_always(5, val));
        }
        chunks.push(encode_string_field(6, &self.readme_file_path));
        if self.package_type != 0 {
            chunks.push(encode_varint_field_always(7, self.package_type as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.name = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.description = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.folder_path = String::from_utf8(field.value).ok()?;
                }
                4 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.enabled = val != 0;
                }
                5 => {
                    msg.parse_error = Some(String::from_utf8(field.value).ok()?);
                }
                6 => {
                    msg.readme_file_path = String::from_utf8(field.value).ok()?;
                }
                7 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.package_type = val as i32;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SkillOptions {
    pub skill_descriptors: Vec<SkillDescriptor>,
}

impl SkillOptions {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        let items_skill_descriptors: Vec<Vec<u8>> = self
            .skill_descriptors
            .iter()
            .map(|item| item.encode())
            .collect();
        chunks.push(encode_repeated_message_field(1, &items_skill_descriptors));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.skill_descriptors
                        .push(SkillDescriptor::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RequestContext {
    pub rules: Vec<CursorRule>,
    pub env: Option<RequestContextEnv>,
    pub repository_info: Vec<RepositoryIndexingInfo>,
    pub tools: Vec<McpToolDefinition>,
    pub conversation_notes_listing: Option<String>,
    pub shared_notes_listing: Option<String>,
    pub git_repos: Vec<GitRepoInfo>,
    pub project_layouts: Vec<LsDirectoryTreeNode>,
    pub mcp_instructions: Vec<McpInstructions>,
    pub debug_mode_config: Option<DebugModeConfig>,
    pub cloud_rule: Option<String>,
    pub web_search_enabled: Option<bool>,
    pub skill_options: Option<SkillOptions>,
    pub repository_info_should_query_prod: Option<bool>,
    pub file_contents: std::collections::HashMap<String, String>,
    pub user_intent_summary: Option<String>,
    pub custom_subagents: Vec<CustomSubagent>,
    pub mcp_file_system_options: Option<McpFileSystemOptions>,
}

impl RequestContext {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        let items_rules: Vec<Vec<u8>> = self.rules.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(2, &items_rules));
        if let Some(ref val) = self.env {
            chunks.push(encode_message_field(4, &val.encode()));
        }
        let items_repository_info: Vec<Vec<u8>> = self
            .repository_info
            .iter()
            .map(|item| item.encode())
            .collect();
        chunks.push(encode_repeated_message_field(6, &items_repository_info));
        let items_tools: Vec<Vec<u8>> = self.tools.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(7, &items_tools));
        if let Some(ref val) = self.conversation_notes_listing {
            chunks.push(encode_string_field_always(8, val));
        }
        if let Some(ref val) = self.shared_notes_listing {
            chunks.push(encode_string_field_always(9, val));
        }
        let items_git_repos: Vec<Vec<u8>> =
            self.git_repos.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(11, &items_git_repos));
        let items_project_layouts: Vec<Vec<u8>> = self
            .project_layouts
            .iter()
            .map(|item| item.encode())
            .collect();
        chunks.push(encode_repeated_message_field(13, &items_project_layouts));
        let items_mcp_instructions: Vec<Vec<u8>> = self
            .mcp_instructions
            .iter()
            .map(|item| item.encode())
            .collect();
        chunks.push(encode_repeated_message_field(14, &items_mcp_instructions));
        if let Some(ref val) = self.debug_mode_config {
            chunks.push(encode_message_field(15, &val.encode()));
        }
        if let Some(ref val) = self.cloud_rule {
            chunks.push(encode_string_field_always(16, val));
        }
        if let Some(ref val) = self.web_search_enabled {
            chunks.push(encode_bool_field_always(17, *val));
        }
        if let Some(ref val) = self.skill_options {
            chunks.push(encode_message_field(18, &val.encode()));
        }
        if let Some(ref val) = self.repository_info_should_query_prod {
            chunks.push(encode_bool_field_always(19, *val));
        }
        for (key, val) in &self.file_contents {
            let mut entry_chunks = Vec::new();
            entry_chunks.push(encode_string_field(1, key));
            entry_chunks.push(encode_string_field_always(2, val));
            chunks.push(encode_message_field(20, &concat_bytes(&entry_chunks)));
        }
        if let Some(ref val) = self.user_intent_summary {
            chunks.push(encode_string_field_always(21, val));
        }
        let items_custom_subagents: Vec<Vec<u8>> = self
            .custom_subagents
            .iter()
            .map(|item| item.encode())
            .collect();
        chunks.push(encode_repeated_message_field(22, &items_custom_subagents));
        if let Some(ref val) = self.mcp_file_system_options {
            chunks.push(encode_message_field(23, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                2 => {
                    msg.rules.push(CursorRule::decode(&field.value)?);
                }
                4 => {
                    msg.env = Some(RequestContextEnv::decode(&field.value)?);
                }
                6 => {
                    msg.repository_info
                        .push(RepositoryIndexingInfo::decode(&field.value)?);
                }
                7 => {
                    msg.tools.push(McpToolDefinition::decode(&field.value)?);
                }
                8 => {
                    msg.conversation_notes_listing = Some(String::from_utf8(field.value).ok()?);
                }
                9 => {
                    msg.shared_notes_listing = Some(String::from_utf8(field.value).ok()?);
                }
                11 => {
                    msg.git_repos.push(GitRepoInfo::decode(&field.value)?);
                }
                13 => {
                    msg.project_layouts
                        .push(LsDirectoryTreeNode::decode(&field.value)?);
                }
                14 => {
                    msg.mcp_instructions
                        .push(McpInstructions::decode(&field.value)?);
                }
                15 => {
                    msg.debug_mode_config = Some(DebugModeConfig::decode(&field.value)?);
                }
                16 => {
                    msg.cloud_rule = Some(String::from_utf8(field.value).ok()?);
                }
                17 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.web_search_enabled = Some(val != 0);
                }
                18 => {
                    msg.skill_options = Some(SkillOptions::decode(&field.value)?);
                }
                19 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.repository_info_should_query_prod = Some(val != 0);
                }
                20 => {
                    let mut entry_key = String::new();
                    let mut entry_value = <String>::default();
                    for sub in parse_proto_fields(&field.value) {
                        match sub.number {
                            1 => entry_key = String::from_utf8(sub.value).ok()?,
                            2 => {
                                entry_value = String::from_utf8(sub.value).ok()?;
                            }
                            _ => {}
                        }
                    }
                    msg.file_contents.insert(entry_key, entry_value);
                }
                21 => {
                    msg.user_intent_summary = Some(String::from_utf8(field.value).ok()?);
                }
                22 => {
                    msg.custom_subagents
                        .push(CustomSubagent::decode(&field.value)?);
                }
                23 => {
                    msg.mcp_file_system_options = Some(McpFileSystemOptions::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SandboxPolicy {
    pub type_: i32,
    pub network_access: Option<bool>,
    pub additional_readwrite_paths: Vec<String>,
    pub additional_readonly_paths: Vec<String>,
    pub debug_output_dir: Option<String>,
    pub block_git_writes: Option<bool>,
    pub disable_tmp_write: Option<bool>,
}

impl SandboxPolicy {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.type_ != 0 {
            chunks.push(encode_varint_field_always(1, self.type_ as u64));
        }
        if let Some(ref val) = self.network_access {
            chunks.push(encode_bool_field_always(2, *val));
        }
        chunks.push(encode_repeated_string_field(
            3,
            &self.additional_readwrite_paths,
        ));
        chunks.push(encode_repeated_string_field(
            4,
            &self.additional_readonly_paths,
        ));
        if let Some(ref val) = self.debug_output_dir {
            chunks.push(encode_string_field_always(5, val));
        }
        if let Some(ref val) = self.block_git_writes {
            chunks.push(encode_bool_field_always(6, *val));
        }
        if let Some(ref val) = self.disable_tmp_write {
            chunks.push(encode_bool_field_always(7, *val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.type_ = val as i32;
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.network_access = Some(val != 0);
                }
                3 => {
                    msg.additional_readwrite_paths
                        .push(String::from_utf8(field.value).ok()?);
                }
                4 => {
                    msg.additional_readonly_paths
                        .push(String::from_utf8(field.value).ok()?);
                }
                5 => {
                    msg.debug_output_dir = Some(String::from_utf8(field.value).ok()?);
                }
                6 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.block_git_writes = Some(val != 0);
                }
                7 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.disable_tmp_write = Some(val != 0);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectedImageDataOrBlobId {
    BlobId(Vec<u8>),
    Data(Vec<u8>),
    BlobIdWithData(SelectedImage_BlobIdWithData),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SelectedImage {
    pub uuid: String,
    pub path: String,
    pub dimension: Option<SelectedImage_Dimension>,
    pub mime_type: String,
    pub data_or_blob_id: Option<SelectedImageDataOrBlobId>,
}

impl SelectedImage {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(2, &self.uuid));
        chunks.push(encode_string_field(3, &self.path));
        if let Some(ref val) = self.dimension {
            chunks.push(encode_message_field(4, &val.encode()));
        }
        chunks.push(encode_string_field(7, &self.mime_type));
        if let Some(ref val) = self.data_or_blob_id {
            match val {
                SelectedImageDataOrBlobId::BlobId(ref inner) => {
                    chunks.push(encode_message_field(1, inner));
                }
                SelectedImageDataOrBlobId::Data(ref inner) => {
                    chunks.push(encode_message_field(8, inner));
                }
                SelectedImageDataOrBlobId::BlobIdWithData(ref inner) => {
                    chunks.push(encode_message_field(9, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                2 => {
                    msg.uuid = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                4 => {
                    msg.dimension = Some(SelectedImage_Dimension::decode(&field.value)?);
                }
                7 => {
                    msg.mime_type = String::from_utf8(field.value).ok()?;
                }
                1 => {
                    msg.data_or_blob_id = Some(SelectedImageDataOrBlobId::BlobId(field.value));
                }
                8 => {
                    msg.data_or_blob_id = Some(SelectedImageDataOrBlobId::Data(field.value));
                }
                9 => {
                    let val = SelectedImage_BlobIdWithData::decode(&field.value)?;
                    msg.data_or_blob_id = Some(SelectedImageDataOrBlobId::BlobIdWithData(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SelectedImage_BlobIdWithData {
    pub blob_id: Vec<u8>,
    pub data: Vec<u8>,
}

impl SelectedImage_BlobIdWithData {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if !self.blob_id.is_empty() {
            chunks.push(encode_message_field(1, &self.blob_id));
        }
        if !self.data.is_empty() {
            chunks.push(encode_message_field(2, &self.data));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.blob_id = field.value;
                }
                2 => {
                    msg.data = field.value;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SelectedImage_Dimension {
    pub width: i32,
    pub height: i32,
}

impl SelectedImage_Dimension {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.width != 0 {
            chunks.push(encode_varint_field_always(1, self.width as u64));
        }
        if self.height != 0 {
            chunks.push(encode_varint_field_always(2, self.height as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.width = val as i32;
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.height = val as i32;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExtraContextEntryDataOrBlobId {
    Data(String),
    BlobId(Vec<u8>),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExtraContextEntry {
    pub data_or_blob_id: Option<ExtraContextEntryDataOrBlobId>,
}

impl ExtraContextEntry {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.data_or_blob_id {
            match val {
                ExtraContextEntryDataOrBlobId::Data(ref inner) => {
                    chunks.push(encode_string_field_always(1, inner));
                }
                ExtraContextEntryDataOrBlobId::BlobId(ref inner) => {
                    chunks.push(encode_message_field(2, inner));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = String::from_utf8(field.value).ok()?;
                    msg.data_or_blob_id = Some(ExtraContextEntryDataOrBlobId::Data(val));
                }
                2 => {
                    msg.data_or_blob_id = Some(ExtraContextEntryDataOrBlobId::BlobId(field.value));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SelectedFile {
    pub content: String,
    pub path: String,
    pub relative_path: Option<String>,
}

impl SelectedFile {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.content));
        chunks.push(encode_string_field(2, &self.path));
        if let Some(ref val) = self.relative_path {
            chunks.push(encode_string_field_always(3, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.content = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.relative_path = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SelectedCodeSelection {
    pub content: String,
    pub path: String,
    pub relative_path: Option<String>,
    pub range: Option<Range>,
}

impl SelectedCodeSelection {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.content));
        chunks.push(encode_string_field(2, &self.path));
        if let Some(ref val) = self.relative_path {
            chunks.push(encode_string_field_always(3, val));
        }
        if let Some(ref val) = self.range {
            chunks.push(encode_message_field(4, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.content = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.relative_path = Some(String::from_utf8(field.value).ok()?);
                }
                4 => {
                    msg.range = Some(Range::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SelectedTerminal {
    pub content: String,
    pub title: Option<String>,
    pub path: Option<String>,
}

impl SelectedTerminal {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.content));
        if let Some(ref val) = self.title {
            chunks.push(encode_string_field_always(2, val));
        }
        if let Some(ref val) = self.path {
            chunks.push(encode_string_field_always(3, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.content = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.title = Some(String::from_utf8(field.value).ok()?);
                }
                3 => {
                    msg.path = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SelectedTerminalSelection {
    pub content: String,
    pub title: Option<String>,
    pub path: Option<String>,
    pub range: Option<Range>,
}

impl SelectedTerminalSelection {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.content));
        if let Some(ref val) = self.title {
            chunks.push(encode_string_field_always(2, val));
        }
        if let Some(ref val) = self.path {
            chunks.push(encode_string_field_always(3, val));
        }
        if let Some(ref val) = self.range {
            chunks.push(encode_message_field(4, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.content = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.title = Some(String::from_utf8(field.value).ok()?);
                }
                3 => {
                    msg.path = Some(String::from_utf8(field.value).ok()?);
                }
                4 => {
                    msg.range = Some(Range::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SelectedFolder {
    pub path: String,
    pub relative_path: Option<String>,
    pub directory_tree: Option<LsDirectoryTreeNode>,
}

impl SelectedFolder {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        if let Some(ref val) = self.relative_path {
            chunks.push(encode_string_field_always(2, val));
        }
        if let Some(ref val) = self.directory_tree {
            chunks.push(encode_message_field(3, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.relative_path = Some(String::from_utf8(field.value).ok()?);
                }
                3 => {
                    msg.directory_tree = Some(LsDirectoryTreeNode::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SelectedExternalLink {
    pub url: String,
    pub uuid: String,
    pub pdf_content: Option<String>,
    pub is_pdf: Option<bool>,
    pub filename: Option<String>,
}

impl SelectedExternalLink {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.url));
        chunks.push(encode_string_field(2, &self.uuid));
        if let Some(ref val) = self.pdf_content {
            chunks.push(encode_string_field_always(3, val));
        }
        if let Some(ref val) = self.is_pdf {
            chunks.push(encode_bool_field_always(4, *val));
        }
        if let Some(ref val) = self.filename {
            chunks.push(encode_string_field_always(5, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.url = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.uuid = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.pdf_content = Some(String::from_utf8(field.value).ok()?);
                }
                4 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.is_pdf = Some(val != 0);
                }
                5 => {
                    msg.filename = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SelectedCursorRule {
    pub rule: Option<CursorRule>,
}

impl SelectedCursorRule {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.rule {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.rule = Some(CursorRule::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SelectedGitDiff {
    pub content: String,
}

impl SelectedGitDiff {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.content));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.content = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SelectedGitDiffFromBranchToMain {
    pub content: String,
}

impl SelectedGitDiffFromBranchToMain {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.content));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.content = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SelectedGitCommit {
    pub sha: String,
    pub message: String,
    pub description: Option<String>,
    pub diff: String,
}

impl SelectedGitCommit {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.sha));
        chunks.push(encode_string_field(2, &self.message));
        if let Some(ref val) = self.description {
            chunks.push(encode_string_field_always(3, val));
        }
        chunks.push(encode_string_field(4, &self.diff));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.sha = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.message = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.description = Some(String::from_utf8(field.value).ok()?);
                }
                4 => {
                    msg.diff = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SelectedPullRequest {
    pub number: i32,
    pub url: String,
    pub title: Option<String>,
    pub folder_path: String,
    pub summary_json: Option<String>,
    pub description: Option<String>,
    pub blob_id: Option<Vec<u8>>,
}

impl SelectedPullRequest {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.number != 0 {
            chunks.push(encode_varint_field_always(1, self.number as u64));
        }
        chunks.push(encode_string_field(2, &self.url));
        if let Some(ref val) = self.title {
            chunks.push(encode_string_field_always(3, val));
        }
        chunks.push(encode_string_field(4, &self.folder_path));
        if let Some(ref val) = self.summary_json {
            chunks.push(encode_string_field_always(5, val));
        }
        if let Some(ref val) = self.description {
            chunks.push(encode_string_field_always(6, val));
        }
        if let Some(ref val) = self.blob_id {
            chunks.push(encode_message_field(7, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.number = val as i32;
                }
                2 => {
                    msg.url = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.title = Some(String::from_utf8(field.value).ok()?);
                }
                4 => {
                    msg.folder_path = String::from_utf8(field.value).ok()?;
                }
                5 => {
                    msg.summary_json = Some(String::from_utf8(field.value).ok()?);
                }
                6 => {
                    msg.description = Some(String::from_utf8(field.value).ok()?);
                }
                7 => {
                    msg.blob_id = Some(field.value);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SelectedGitPRDiffSelection {
    pub pr_url: String,
    pub file_path: String,
    pub start_line: i32,
    pub end_line: i32,
    pub diff_content: Option<String>,
    pub blob_id: Option<Vec<u8>>,
}

impl SelectedGitPRDiffSelection {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.pr_url));
        chunks.push(encode_string_field(2, &self.file_path));
        if self.start_line != 0 {
            chunks.push(encode_varint_field_always(3, self.start_line as u64));
        }
        if self.end_line != 0 {
            chunks.push(encode_varint_field_always(4, self.end_line as u64));
        }
        if let Some(ref val) = self.diff_content {
            chunks.push(encode_string_field_always(5, val));
        }
        if let Some(ref val) = self.blob_id {
            chunks.push(encode_message_field(6, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.pr_url = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.file_path = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.start_line = val as i32;
                }
                4 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.end_line = val as i32;
                }
                5 => {
                    msg.diff_content = Some(String::from_utf8(field.value).ok()?);
                }
                6 => {
                    msg.blob_id = Some(field.value);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SelectedCursorCommand {
    pub name: String,
    pub content: String,
}

impl SelectedCursorCommand {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.name));
        chunks.push(encode_string_field(2, &self.content));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.name = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.content = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SelectedDocumentation {
    pub doc_id: String,
    pub name: String,
}

impl SelectedDocumentation {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.doc_id));
        chunks.push(encode_string_field(2, &self.name));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.doc_id = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.name = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SelectedPastChat {
    pub agent_id: String,
    pub name: String,
}

impl SelectedPastChat {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.agent_id));
        chunks.push(encode_string_field(2, &self.name));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.agent_id = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.name = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CallFrame {
    pub function_name: Option<String>,
    pub url: Option<String>,
    pub line_number: Option<i32>,
    pub column_number: Option<i32>,
}

impl CallFrame {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.function_name {
            chunks.push(encode_string_field_always(1, val));
        }
        if let Some(ref val) = self.url {
            chunks.push(encode_string_field_always(2, val));
        }
        if let Some(ref val) = self.line_number {
            chunks.push(encode_varint_field_always(3, *val as u64));
        }
        if let Some(ref val) = self.column_number {
            chunks.push(encode_varint_field_always(4, *val as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.function_name = Some(String::from_utf8(field.value).ok()?);
                }
                2 => {
                    msg.url = Some(String::from_utf8(field.value).ok()?);
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.line_number = Some(val as i32);
                }
                4 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.column_number = Some(val as i32);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct StackTrace {
    pub call_frames: Vec<CallFrame>,
    pub raw_stack_trace: Option<String>,
}

impl StackTrace {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        let items_call_frames: Vec<Vec<u8>> =
            self.call_frames.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(1, &items_call_frames));
        if let Some(ref val) = self.raw_stack_trace {
            chunks.push(encode_string_field_always(2, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.call_frames.push(CallFrame::decode(&field.value)?);
                }
                2 => {
                    msg.raw_stack_trace = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SelectedConsoleLog {
    pub message: String,
    pub timestamp: f64,
    pub level: String,
    pub client_name: String,
    pub session_id: String,
    pub stack_trace: Option<StackTrace>,
    pub object_data_json: Option<String>,
}

impl SelectedConsoleLog {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.message));
        if self.timestamp != 0.0 {
            chunks.push(encode_double_field_always(2, self.timestamp));
        }
        chunks.push(encode_string_field(3, &self.level));
        chunks.push(encode_string_field(4, &self.client_name));
        chunks.push(encode_string_field(5, &self.session_id));
        if let Some(ref val) = self.stack_trace {
            chunks.push(encode_message_field(6, &val.encode()));
        }
        if let Some(ref val) = self.object_data_json {
            chunks.push(encode_string_field_always(7, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.message = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.timestamp = f64::from_le_bytes(field.value.try_into().ok()?);
                }
                3 => {
                    msg.level = String::from_utf8(field.value).ok()?;
                }
                4 => {
                    msg.client_name = String::from_utf8(field.value).ok()?;
                }
                5 => {
                    msg.session_id = String::from_utf8(field.value).ok()?;
                }
                6 => {
                    msg.stack_trace = Some(StackTrace::decode(&field.value)?);
                }
                7 => {
                    msg.object_data_json = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SelectedUIElement {
    pub element: String,
    pub xpath: String,
    pub text_content: String,
    pub extra: String,
    pub component: Option<String>,
    pub component_props_json: Option<String>,
}

impl SelectedUIElement {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.element));
        chunks.push(encode_string_field(2, &self.xpath));
        chunks.push(encode_string_field(3, &self.text_content));
        chunks.push(encode_string_field(4, &self.extra));
        if let Some(ref val) = self.component {
            chunks.push(encode_string_field_always(5, val));
        }
        if let Some(ref val) = self.component_props_json {
            chunks.push(encode_string_field_always(6, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.element = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.xpath = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.text_content = String::from_utf8(field.value).ok()?;
                }
                4 => {
                    msg.extra = String::from_utf8(field.value).ok()?;
                }
                5 => {
                    msg.component = Some(String::from_utf8(field.value).ok()?);
                }
                6 => {
                    msg.component_props_json = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SelectedSubagent {
    pub name: String,
}

impl SelectedSubagent {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.name));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.name = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SelectedContext {
    pub selected_images: Vec<SelectedImage>,
    pub invocation_context: Option<InvocationContext>,
    pub extra_context: Vec<String>,
    pub extra_context_entries: Vec<ExtraContextEntry>,
    pub files: Vec<SelectedFile>,
    pub code_selections: Vec<SelectedCodeSelection>,
    pub terminals: Vec<SelectedTerminal>,
    pub terminal_selections: Vec<SelectedTerminalSelection>,
    pub folders: Vec<SelectedFolder>,
    pub external_links: Vec<SelectedExternalLink>,
    pub cursor_rules: Vec<SelectedCursorRule>,
    pub git_diff: Option<SelectedGitDiff>,
    pub git_diff_from_branch_to_main: Option<SelectedGitDiffFromBranchToMain>,
    pub cursor_commands: Vec<SelectedCursorCommand>,
    pub documentations: Vec<SelectedDocumentation>,
    pub ui_elements: Vec<SelectedUIElement>,
    pub console_logs: Vec<SelectedConsoleLog>,
    pub git_commits: Vec<SelectedGitCommit>,
    pub past_chats: Vec<SelectedPastChat>,
    pub git_pr_diff_selections: Vec<SelectedGitPRDiffSelection>,
    pub selected_pull_requests: Vec<SelectedPullRequest>,
    pub selected_subagents: Vec<SelectedSubagent>,
}

impl SelectedContext {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        let items_selected_images: Vec<Vec<u8>> = self
            .selected_images
            .iter()
            .map(|item| item.encode())
            .collect();
        chunks.push(encode_repeated_message_field(1, &items_selected_images));
        if let Some(ref val) = self.invocation_context {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        chunks.push(encode_repeated_string_field(3, &self.extra_context));
        let items_extra_context_entries: Vec<Vec<u8>> = self
            .extra_context_entries
            .iter()
            .map(|item| item.encode())
            .collect();
        chunks.push(encode_repeated_message_field(
            16,
            &items_extra_context_entries,
        ));
        let items_files: Vec<Vec<u8>> = self.files.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(4, &items_files));
        let items_code_selections: Vec<Vec<u8>> = self
            .code_selections
            .iter()
            .map(|item| item.encode())
            .collect();
        chunks.push(encode_repeated_message_field(5, &items_code_selections));
        let items_terminals: Vec<Vec<u8>> =
            self.terminals.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(6, &items_terminals));
        let items_terminal_selections: Vec<Vec<u8>> = self
            .terminal_selections
            .iter()
            .map(|item| item.encode())
            .collect();
        chunks.push(encode_repeated_message_field(7, &items_terminal_selections));
        let items_folders: Vec<Vec<u8>> = self.folders.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(8, &items_folders));
        let items_external_links: Vec<Vec<u8>> = self
            .external_links
            .iter()
            .map(|item| item.encode())
            .collect();
        chunks.push(encode_repeated_message_field(9, &items_external_links));
        let items_cursor_rules: Vec<Vec<u8>> =
            self.cursor_rules.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(10, &items_cursor_rules));
        if let Some(ref val) = self.git_diff {
            chunks.push(encode_message_field(18, &val.encode()));
        }
        if let Some(ref val) = self.git_diff_from_branch_to_main {
            chunks.push(encode_message_field(11, &val.encode()));
        }
        let items_cursor_commands: Vec<Vec<u8>> = self
            .cursor_commands
            .iter()
            .map(|item| item.encode())
            .collect();
        chunks.push(encode_repeated_message_field(12, &items_cursor_commands));
        let items_documentations: Vec<Vec<u8>> = self
            .documentations
            .iter()
            .map(|item| item.encode())
            .collect();
        chunks.push(encode_repeated_message_field(13, &items_documentations));
        let items_ui_elements: Vec<Vec<u8>> =
            self.ui_elements.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(14, &items_ui_elements));
        let items_console_logs: Vec<Vec<u8>> =
            self.console_logs.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(15, &items_console_logs));
        let items_git_commits: Vec<Vec<u8>> =
            self.git_commits.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(17, &items_git_commits));
        let items_past_chats: Vec<Vec<u8>> =
            self.past_chats.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(19, &items_past_chats));
        let items_git_pr_diff_selections: Vec<Vec<u8>> = self
            .git_pr_diff_selections
            .iter()
            .map(|item| item.encode())
            .collect();
        chunks.push(encode_repeated_message_field(
            20,
            &items_git_pr_diff_selections,
        ));
        let items_selected_pull_requests: Vec<Vec<u8>> = self
            .selected_pull_requests
            .iter()
            .map(|item| item.encode())
            .collect();
        chunks.push(encode_repeated_message_field(
            21,
            &items_selected_pull_requests,
        ));
        let items_selected_subagents: Vec<Vec<u8>> = self
            .selected_subagents
            .iter()
            .map(|item| item.encode())
            .collect();
        chunks.push(encode_repeated_message_field(22, &items_selected_subagents));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.selected_images
                        .push(SelectedImage::decode(&field.value)?);
                }
                2 => {
                    msg.invocation_context = Some(InvocationContext::decode(&field.value)?);
                }
                3 => {
                    msg.extra_context.push(String::from_utf8(field.value).ok()?);
                }
                16 => {
                    msg.extra_context_entries
                        .push(ExtraContextEntry::decode(&field.value)?);
                }
                4 => {
                    msg.files.push(SelectedFile::decode(&field.value)?);
                }
                5 => {
                    msg.code_selections
                        .push(SelectedCodeSelection::decode(&field.value)?);
                }
                6 => {
                    msg.terminals.push(SelectedTerminal::decode(&field.value)?);
                }
                7 => {
                    msg.terminal_selections
                        .push(SelectedTerminalSelection::decode(&field.value)?);
                }
                8 => {
                    msg.folders.push(SelectedFolder::decode(&field.value)?);
                }
                9 => {
                    msg.external_links
                        .push(SelectedExternalLink::decode(&field.value)?);
                }
                10 => {
                    msg.cursor_rules
                        .push(SelectedCursorRule::decode(&field.value)?);
                }
                18 => {
                    msg.git_diff = Some(SelectedGitDiff::decode(&field.value)?);
                }
                11 => {
                    msg.git_diff_from_branch_to_main =
                        Some(SelectedGitDiffFromBranchToMain::decode(&field.value)?);
                }
                12 => {
                    msg.cursor_commands
                        .push(SelectedCursorCommand::decode(&field.value)?);
                }
                13 => {
                    msg.documentations
                        .push(SelectedDocumentation::decode(&field.value)?);
                }
                14 => {
                    msg.ui_elements
                        .push(SelectedUIElement::decode(&field.value)?);
                }
                15 => {
                    msg.console_logs
                        .push(SelectedConsoleLog::decode(&field.value)?);
                }
                17 => {
                    msg.git_commits
                        .push(SelectedGitCommit::decode(&field.value)?);
                }
                19 => {
                    msg.past_chats.push(SelectedPastChat::decode(&field.value)?);
                }
                20 => {
                    msg.git_pr_diff_selections
                        .push(SelectedGitPRDiffSelection::decode(&field.value)?);
                }
                21 => {
                    msg.selected_pull_requests
                        .push(SelectedPullRequest::decode(&field.value)?);
                }
                22 => {
                    msg.selected_subagents
                        .push(SelectedSubagent::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InvocationContextData {
    SlackThread(InvocationContext_SlackThread),
    GithubPr(InvocationContext_GithubPR),
    IdeState(InvocationContext_IdeState),
    BlobId(Vec<u8>),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct InvocationContext {
    pub data: Option<InvocationContextData>,
}

impl InvocationContext {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.data {
            match val {
                InvocationContextData::SlackThread(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                InvocationContextData::GithubPr(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                InvocationContextData::IdeState(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
                InvocationContextData::BlobId(ref inner) => {
                    chunks.push(encode_message_field(10, inner));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = InvocationContext_SlackThread::decode(&field.value)?;
                    msg.data = Some(InvocationContextData::SlackThread(val));
                }
                2 => {
                    let val = InvocationContext_GithubPR::decode(&field.value)?;
                    msg.data = Some(InvocationContextData::GithubPr(val));
                }
                3 => {
                    let val = InvocationContext_IdeState::decode(&field.value)?;
                    msg.data = Some(InvocationContextData::IdeState(val));
                }
                10 => {
                    msg.data = Some(InvocationContextData::BlobId(field.value));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct InvocationContext_SlackThread {
    pub thread: String,
    pub channel_name: Option<String>,
    pub channel_purpose: Option<String>,
    pub channel_topic: Option<String>,
}

impl InvocationContext_SlackThread {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.thread));
        if let Some(ref val) = self.channel_name {
            chunks.push(encode_string_field_always(2, val));
        }
        if let Some(ref val) = self.channel_purpose {
            chunks.push(encode_string_field_always(3, val));
        }
        if let Some(ref val) = self.channel_topic {
            chunks.push(encode_string_field_always(4, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.thread = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.channel_name = Some(String::from_utf8(field.value).ok()?);
                }
                3 => {
                    msg.channel_purpose = Some(String::from_utf8(field.value).ok()?);
                }
                4 => {
                    msg.channel_topic = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct InvocationContext_GithubPR {
    pub title: String,
    pub description: String,
    pub comments: String,
    pub ci_failures: Option<String>,
}

impl InvocationContext_GithubPR {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.title));
        chunks.push(encode_string_field(2, &self.description));
        chunks.push(encode_string_field(3, &self.comments));
        if let Some(ref val) = self.ci_failures {
            chunks.push(encode_string_field_always(4, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.title = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.description = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.comments = String::from_utf8(field.value).ok()?;
                }
                4 => {
                    msg.ci_failures = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct InvocationContext_IdeState {
    pub visible_files: Vec<InvocationContext_IdeState_File>,
    pub recently_viewed_files: Vec<InvocationContext_IdeState_File>,
    pub currently_viewed_prs: Vec<InvocationContext_IdeState_ViewedPullRequest>,
}

impl InvocationContext_IdeState {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        let items_visible_files: Vec<Vec<u8>> = self
            .visible_files
            .iter()
            .map(|item| item.encode())
            .collect();
        chunks.push(encode_repeated_message_field(1, &items_visible_files));
        let items_recently_viewed_files: Vec<Vec<u8>> = self
            .recently_viewed_files
            .iter()
            .map(|item| item.encode())
            .collect();
        chunks.push(encode_repeated_message_field(
            2,
            &items_recently_viewed_files,
        ));
        let items_currently_viewed_prs: Vec<Vec<u8>> = self
            .currently_viewed_prs
            .iter()
            .map(|item| item.encode())
            .collect();
        chunks.push(encode_repeated_message_field(
            3,
            &items_currently_viewed_prs,
        ));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.visible_files
                        .push(InvocationContext_IdeState_File::decode(&field.value)?);
                }
                2 => {
                    msg.recently_viewed_files
                        .push(InvocationContext_IdeState_File::decode(&field.value)?);
                }
                3 => {
                    msg.currently_viewed_prs.push(
                        InvocationContext_IdeState_ViewedPullRequest::decode(&field.value)?,
                    );
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct InvocationContext_IdeState_File {
    pub path: String,
    pub relative_path: Option<String>,
    pub cursor_position: Option<InvocationContext_IdeState_File_CursorPosition>,
    pub total_lines: i32,
    pub active_command: Option<String>,
}

impl InvocationContext_IdeState_File {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        if let Some(ref val) = self.relative_path {
            chunks.push(encode_string_field_always(2, val));
        }
        if let Some(ref val) = self.cursor_position {
            chunks.push(encode_message_field(3, &val.encode()));
        }
        if self.total_lines != 0 {
            chunks.push(encode_varint_field_always(4, self.total_lines as u64));
        }
        if let Some(ref val) = self.active_command {
            chunks.push(encode_string_field_always(5, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.relative_path = Some(String::from_utf8(field.value).ok()?);
                }
                3 => {
                    msg.cursor_position = Some(
                        InvocationContext_IdeState_File_CursorPosition::decode(&field.value)?,
                    );
                }
                4 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.total_lines = val as i32;
                }
                5 => {
                    msg.active_command = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct InvocationContext_IdeState_File_CursorPosition {
    pub line: i32,
    pub text: String,
}

impl InvocationContext_IdeState_File_CursorPosition {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.line != 0 {
            chunks.push(encode_varint_field_always(1, self.line as u64));
        }
        chunks.push(encode_string_field(2, &self.text));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.line = val as i32;
                }
                2 => {
                    msg.text = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct InvocationContext_IdeState_ViewedPullRequest {
    pub number: i32,
    pub url: String,
    pub title: Option<String>,
    pub folder_path: Option<String>,
    pub summary_json: Option<String>,
    pub description: Option<String>,
}

impl InvocationContext_IdeState_ViewedPullRequest {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.number != 0 {
            chunks.push(encode_varint_field_always(1, self.number as u64));
        }
        chunks.push(encode_string_field(2, &self.url));
        if let Some(ref val) = self.title {
            chunks.push(encode_string_field_always(3, val));
        }
        if let Some(ref val) = self.folder_path {
            chunks.push(encode_string_field_always(4, val));
        }
        if let Some(ref val) = self.summary_json {
            chunks.push(encode_string_field_always(5, val));
        }
        if let Some(ref val) = self.description {
            chunks.push(encode_string_field_always(6, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.number = val as i32;
                }
                2 => {
                    msg.url = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.title = Some(String::from_utf8(field.value).ok()?);
                }
                4 => {
                    msg.folder_path = Some(String::from_utf8(field.value).ok()?);
                }
                5 => {
                    msg.summary_json = Some(String::from_utf8(field.value).ok()?);
                }
                6 => {
                    msg.description = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SetupVmEnvironmentArgs {
    pub install_command: String,
    pub start_command: String,
}

impl SetupVmEnvironmentArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(2, &self.install_command));
        chunks.push(encode_string_field(3, &self.start_command));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                2 => {
                    msg.install_command = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.start_command = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SetupVmEnvironmentResultResult {
    Success(SetupVmEnvironmentSuccess),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SetupVmEnvironmentResult {
    pub result: Option<SetupVmEnvironmentResultResult>,
}

impl SetupVmEnvironmentResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                SetupVmEnvironmentResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = SetupVmEnvironmentSuccess::decode(&field.value)?;
                    msg.result = Some(SetupVmEnvironmentResultResult::Success(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SetupVmEnvironmentSuccess {}

impl SetupVmEnvironmentSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SetupVmEnvironmentToolCall {
    pub args: Option<SetupVmEnvironmentArgs>,
    pub result: Option<SetupVmEnvironmentResult>,
}

impl SetupVmEnvironmentToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(SetupVmEnvironmentArgs::decode(&field.value)?);
                }
                2 => {
                    msg.result = Some(SetupVmEnvironmentResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShellCommandParsingResult {
    pub parsing_failed: bool,
    pub executable_commands: Vec<ShellCommandParsingResult_ExecutableCommand>,
    pub has_redirects: bool,
    pub has_command_substitution: bool,
}

impl ShellCommandParsingResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_bool_field(1, self.parsing_failed));
        let items_executable_commands: Vec<Vec<u8>> = self
            .executable_commands
            .iter()
            .map(|item| item.encode())
            .collect();
        chunks.push(encode_repeated_message_field(2, &items_executable_commands));
        chunks.push(encode_bool_field(3, self.has_redirects));
        chunks.push(encode_bool_field(4, self.has_command_substitution));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.parsing_failed = val != 0;
                }
                2 => {
                    msg.executable_commands.push(
                        ShellCommandParsingResult_ExecutableCommand::decode(&field.value)?,
                    );
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.has_redirects = val != 0;
                }
                4 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.has_command_substitution = val != 0;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShellCommandParsingResult_ExecutableCommandArg {
    pub type_: String,
    pub value: String,
}

impl ShellCommandParsingResult_ExecutableCommandArg {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.type_));
        chunks.push(encode_string_field(2, &self.value));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.type_ = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.value = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShellCommandParsingResult_ExecutableCommand {
    pub name: String,
    pub args: Vec<ShellCommandParsingResult_ExecutableCommandArg>,
    pub full_text: String,
}

impl ShellCommandParsingResult_ExecutableCommand {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.name));
        let items_args: Vec<Vec<u8>> = self.args.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(2, &items_args));
        chunks.push(encode_string_field(3, &self.full_text));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.name = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.args
                        .push(ShellCommandParsingResult_ExecutableCommandArg::decode(
                            &field.value,
                        )?);
                }
                3 => {
                    msg.full_text = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShellArgs {
    pub command: String,
    pub working_directory: String,
    pub timeout: i32,
    pub tool_call_id: String,
    pub simple_commands: Vec<String>,
    pub has_input_redirect: bool,
    pub has_output_redirect: bool,
    pub parsing_result: Option<ShellCommandParsingResult>,
    pub requested_sandbox_policy: Option<SandboxPolicy>,
    pub file_output_threshold_bytes: Option<u64>,
    pub is_background: bool,
    pub skip_approval: bool,
    pub timeout_behavior: i32,
    pub hard_timeout: Option<i32>,
}

impl ShellArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.command));
        chunks.push(encode_string_field(2, &self.working_directory));
        if self.timeout != 0 {
            chunks.push(encode_varint_field_always(3, self.timeout as u64));
        }
        chunks.push(encode_string_field(4, &self.tool_call_id));
        chunks.push(encode_repeated_string_field(5, &self.simple_commands));
        chunks.push(encode_bool_field(6, self.has_input_redirect));
        chunks.push(encode_bool_field(7, self.has_output_redirect));
        if let Some(ref val) = self.parsing_result {
            chunks.push(encode_message_field(8, &val.encode()));
        }
        if let Some(ref val) = self.requested_sandbox_policy {
            chunks.push(encode_message_field(9, &val.encode()));
        }
        if let Some(ref val) = self.file_output_threshold_bytes {
            chunks.push(encode_varint_field_always(10, *val as u64));
        }
        chunks.push(encode_bool_field(11, self.is_background));
        chunks.push(encode_bool_field(12, self.skip_approval));
        if self.timeout_behavior != 0 {
            chunks.push(encode_varint_field_always(13, self.timeout_behavior as u64));
        }
        if let Some(ref val) = self.hard_timeout {
            chunks.push(encode_varint_field_always(14, *val as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.command = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.working_directory = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.timeout = val as i32;
                }
                4 => {
                    msg.tool_call_id = String::from_utf8(field.value).ok()?;
                }
                5 => {
                    msg.simple_commands
                        .push(String::from_utf8(field.value).ok()?);
                }
                6 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.has_input_redirect = val != 0;
                }
                7 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.has_output_redirect = val != 0;
                }
                8 => {
                    msg.parsing_result = Some(ShellCommandParsingResult::decode(&field.value)?);
                }
                9 => {
                    msg.requested_sandbox_policy = Some(SandboxPolicy::decode(&field.value)?);
                }
                10 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.file_output_threshold_bytes = Some(val as u64);
                }
                11 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.is_background = val != 0;
                }
                12 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.skip_approval = val != 0;
                }
                13 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.timeout_behavior = val as i32;
                }
                14 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.hard_timeout = Some(val as i32);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShellResultResult {
    Success(ShellSuccess),
    Failure(ShellFailure),
    Timeout(ShellTimeout),
    Rejected(ShellRejected),
    SpawnError(ShellSpawnError),
    PermissionDenied(ShellPermissionDenied),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShellResult {
    pub sandbox_policy: Option<SandboxPolicy>,
    pub is_background: Option<bool>,
    pub terminals_folder: Option<String>,
    pub pid: Option<u32>,
    pub result: Option<ShellResultResult>,
}

impl ShellResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.sandbox_policy {
            chunks.push(encode_message_field(101, &val.encode()));
        }
        if let Some(ref val) = self.is_background {
            chunks.push(encode_bool_field_always(102, *val));
        }
        if let Some(ref val) = self.terminals_folder {
            chunks.push(encode_string_field_always(103, val));
        }
        if let Some(ref val) = self.pid {
            chunks.push(encode_varint_field_always(104, *val as u64));
        }
        if let Some(ref val) = self.result {
            match val {
                ShellResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                ShellResultResult::Failure(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                ShellResultResult::Timeout(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
                ShellResultResult::Rejected(ref inner) => {
                    chunks.push(encode_message_field(4, &inner.encode()));
                }
                ShellResultResult::SpawnError(ref inner) => {
                    chunks.push(encode_message_field(5, &inner.encode()));
                }
                ShellResultResult::PermissionDenied(ref inner) => {
                    chunks.push(encode_message_field(7, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                101 => {
                    msg.sandbox_policy = Some(SandboxPolicy::decode(&field.value)?);
                }
                102 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.is_background = Some(val != 0);
                }
                103 => {
                    msg.terminals_folder = Some(String::from_utf8(field.value).ok()?);
                }
                104 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.pid = Some(val as u32);
                }
                1 => {
                    let val = ShellSuccess::decode(&field.value)?;
                    msg.result = Some(ShellResultResult::Success(val));
                }
                2 => {
                    let val = ShellFailure::decode(&field.value)?;
                    msg.result = Some(ShellResultResult::Failure(val));
                }
                3 => {
                    let val = ShellTimeout::decode(&field.value)?;
                    msg.result = Some(ShellResultResult::Timeout(val));
                }
                4 => {
                    let val = ShellRejected::decode(&field.value)?;
                    msg.result = Some(ShellResultResult::Rejected(val));
                }
                5 => {
                    let val = ShellSpawnError::decode(&field.value)?;
                    msg.result = Some(ShellResultResult::SpawnError(val));
                }
                7 => {
                    let val = ShellPermissionDenied::decode(&field.value)?;
                    msg.result = Some(ShellResultResult::PermissionDenied(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShellStreamStdout {
    pub data: String,
}

impl ShellStreamStdout {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.data));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.data = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShellStreamStderr {
    pub data: String,
}

impl ShellStreamStderr {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.data));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.data = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShellStreamExit {
    pub code: u32,
    pub cwd: String,
    pub output_location: Option<OutputLocation>,
    pub aborted: bool,
    pub abort_reason: Option<i32>,
}

impl ShellStreamExit {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.code != 0 {
            chunks.push(encode_varint_field_always(1, self.code as u64));
        }
        chunks.push(encode_string_field(2, &self.cwd));
        if let Some(ref val) = self.output_location {
            chunks.push(encode_message_field(3, &val.encode()));
        }
        chunks.push(encode_bool_field(4, self.aborted));
        if let Some(ref val) = self.abort_reason {
            chunks.push(encode_varint_field_always(5, *val as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.code = val as u32;
                }
                2 => {
                    msg.cwd = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.output_location = Some(OutputLocation::decode(&field.value)?);
                }
                4 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.aborted = val != 0;
                }
                5 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.abort_reason = Some(val as i32);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShellStreamStart {
    pub sandbox_policy: Option<SandboxPolicy>,
}

impl ShellStreamStart {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.sandbox_policy {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.sandbox_policy = Some(SandboxPolicy::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShellStreamBackgrounded {
    pub shell_id: u32,
    pub command: String,
    pub working_directory: String,
    pub pid: Option<u32>,
    pub ms_to_wait: Option<i32>,
}

impl ShellStreamBackgrounded {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.shell_id != 0 {
            chunks.push(encode_varint_field_always(1, self.shell_id as u64));
        }
        chunks.push(encode_string_field(2, &self.command));
        chunks.push(encode_string_field(3, &self.working_directory));
        if let Some(ref val) = self.pid {
            chunks.push(encode_varint_field_always(4, *val as u64));
        }
        if let Some(ref val) = self.ms_to_wait {
            chunks.push(encode_varint_field_always(5, *val as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.shell_id = val as u32;
                }
                2 => {
                    msg.command = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.working_directory = String::from_utf8(field.value).ok()?;
                }
                4 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.pid = Some(val as u32);
                }
                5 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.ms_to_wait = Some(val as i32);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShellStreamEvent {
    Stdout(ShellStreamStdout),
    Stderr(ShellStreamStderr),
    Exit(ShellStreamExit),
    Start(ShellStreamStart),
    Rejected(ShellRejected),
    PermissionDenied(ShellPermissionDenied),
    Backgrounded(ShellStreamBackgrounded),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShellStream {
    pub event: Option<ShellStreamEvent>,
}

impl ShellStream {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.event {
            match val {
                ShellStreamEvent::Stdout(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                ShellStreamEvent::Stderr(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                ShellStreamEvent::Exit(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
                ShellStreamEvent::Start(ref inner) => {
                    chunks.push(encode_message_field(4, &inner.encode()));
                }
                ShellStreamEvent::Rejected(ref inner) => {
                    chunks.push(encode_message_field(5, &inner.encode()));
                }
                ShellStreamEvent::PermissionDenied(ref inner) => {
                    chunks.push(encode_message_field(6, &inner.encode()));
                }
                ShellStreamEvent::Backgrounded(ref inner) => {
                    chunks.push(encode_message_field(7, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = ShellStreamStdout::decode(&field.value)?;
                    msg.event = Some(ShellStreamEvent::Stdout(val));
                }
                2 => {
                    let val = ShellStreamStderr::decode(&field.value)?;
                    msg.event = Some(ShellStreamEvent::Stderr(val));
                }
                3 => {
                    let val = ShellStreamExit::decode(&field.value)?;
                    msg.event = Some(ShellStreamEvent::Exit(val));
                }
                4 => {
                    let val = ShellStreamStart::decode(&field.value)?;
                    msg.event = Some(ShellStreamEvent::Start(val));
                }
                5 => {
                    let val = ShellRejected::decode(&field.value)?;
                    msg.event = Some(ShellStreamEvent::Rejected(val));
                }
                6 => {
                    let val = ShellPermissionDenied::decode(&field.value)?;
                    msg.event = Some(ShellStreamEvent::PermissionDenied(val));
                }
                7 => {
                    let val = ShellStreamBackgrounded::decode(&field.value)?;
                    msg.event = Some(ShellStreamEvent::Backgrounded(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct OutputLocation {
    pub file_path: String,
    pub size_bytes: i64,
    pub line_count: i64,
}

impl OutputLocation {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.file_path));
        if self.size_bytes != 0 {
            chunks.push(encode_varint_field_always(2, self.size_bytes as u64));
        }
        if self.line_count != 0 {
            chunks.push(encode_varint_field_always(3, self.line_count as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.file_path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.size_bytes = val as i64;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.line_count = val as i64;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShellSuccess {
    pub command: String,
    pub working_directory: String,
    pub exit_code: i32,
    pub signal: String,
    pub stdout: String,
    pub stderr: String,
    pub execution_time: i32,
    pub output_location: Option<OutputLocation>,
    pub shell_id: Option<u32>,
    pub interleaved_output: Option<String>,
    pub pid: Option<u32>,
    pub ms_to_wait: Option<i32>,
}

impl ShellSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.command));
        chunks.push(encode_string_field(2, &self.working_directory));
        if self.exit_code != 0 {
            chunks.push(encode_varint_field_always(3, self.exit_code as u64));
        }
        chunks.push(encode_string_field(4, &self.signal));
        chunks.push(encode_string_field(5, &self.stdout));
        chunks.push(encode_string_field(6, &self.stderr));
        if self.execution_time != 0 {
            chunks.push(encode_varint_field_always(7, self.execution_time as u64));
        }
        if let Some(ref val) = self.output_location {
            chunks.push(encode_message_field(8, &val.encode()));
        }
        if let Some(ref val) = self.shell_id {
            chunks.push(encode_varint_field_always(9, *val as u64));
        }
        if let Some(ref val) = self.interleaved_output {
            chunks.push(encode_string_field_always(10, val));
        }
        if let Some(ref val) = self.pid {
            chunks.push(encode_varint_field_always(11, *val as u64));
        }
        if let Some(ref val) = self.ms_to_wait {
            chunks.push(encode_varint_field_always(12, *val as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.command = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.working_directory = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.exit_code = val as i32;
                }
                4 => {
                    msg.signal = String::from_utf8(field.value).ok()?;
                }
                5 => {
                    msg.stdout = String::from_utf8(field.value).ok()?;
                }
                6 => {
                    msg.stderr = String::from_utf8(field.value).ok()?;
                }
                7 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.execution_time = val as i32;
                }
                8 => {
                    msg.output_location = Some(OutputLocation::decode(&field.value)?);
                }
                9 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.shell_id = Some(val as u32);
                }
                10 => {
                    msg.interleaved_output = Some(String::from_utf8(field.value).ok()?);
                }
                11 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.pid = Some(val as u32);
                }
                12 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.ms_to_wait = Some(val as i32);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShellFailure {
    pub command: String,
    pub working_directory: String,
    pub exit_code: i32,
    pub signal: String,
    pub stdout: String,
    pub stderr: String,
    pub execution_time: i32,
    pub output_location: Option<OutputLocation>,
    pub interleaved_output: Option<String>,
    pub abort_reason: Option<i32>,
    pub aborted: bool,
}

impl ShellFailure {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.command));
        chunks.push(encode_string_field(2, &self.working_directory));
        if self.exit_code != 0 {
            chunks.push(encode_varint_field_always(3, self.exit_code as u64));
        }
        chunks.push(encode_string_field(4, &self.signal));
        chunks.push(encode_string_field(5, &self.stdout));
        chunks.push(encode_string_field(6, &self.stderr));
        if self.execution_time != 0 {
            chunks.push(encode_varint_field_always(7, self.execution_time as u64));
        }
        if let Some(ref val) = self.output_location {
            chunks.push(encode_message_field(8, &val.encode()));
        }
        if let Some(ref val) = self.interleaved_output {
            chunks.push(encode_string_field_always(9, val));
        }
        if let Some(ref val) = self.abort_reason {
            chunks.push(encode_varint_field_always(10, *val as u64));
        }
        chunks.push(encode_bool_field(11, self.aborted));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.command = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.working_directory = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.exit_code = val as i32;
                }
                4 => {
                    msg.signal = String::from_utf8(field.value).ok()?;
                }
                5 => {
                    msg.stdout = String::from_utf8(field.value).ok()?;
                }
                6 => {
                    msg.stderr = String::from_utf8(field.value).ok()?;
                }
                7 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.execution_time = val as i32;
                }
                8 => {
                    msg.output_location = Some(OutputLocation::decode(&field.value)?);
                }
                9 => {
                    msg.interleaved_output = Some(String::from_utf8(field.value).ok()?);
                }
                10 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.abort_reason = Some(val as i32);
                }
                11 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.aborted = val != 0;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShellTimeout {
    pub command: String,
    pub working_directory: String,
    pub timeout_ms: i32,
}

impl ShellTimeout {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.command));
        chunks.push(encode_string_field(2, &self.working_directory));
        if self.timeout_ms != 0 {
            chunks.push(encode_varint_field_always(3, self.timeout_ms as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.command = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.working_directory = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.timeout_ms = val as i32;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShellRejected {
    pub command: String,
    pub working_directory: String,
    pub reason: String,
    pub is_readonly: bool,
}

impl ShellRejected {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.command));
        chunks.push(encode_string_field(2, &self.working_directory));
        chunks.push(encode_string_field(3, &self.reason));
        chunks.push(encode_bool_field(4, self.is_readonly));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.command = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.working_directory = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.reason = String::from_utf8(field.value).ok()?;
                }
                4 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.is_readonly = val != 0;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShellPermissionDenied {
    pub command: String,
    pub working_directory: String,
    pub error: String,
    pub is_readonly: bool,
}

impl ShellPermissionDenied {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.command));
        chunks.push(encode_string_field(2, &self.working_directory));
        chunks.push(encode_string_field(3, &self.error));
        chunks.push(encode_bool_field(4, self.is_readonly));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.command = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.working_directory = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                4 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.is_readonly = val != 0;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShellSpawnError {
    pub command: String,
    pub working_directory: String,
    pub error: String,
}

impl ShellSpawnError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.command));
        chunks.push(encode_string_field(2, &self.working_directory));
        chunks.push(encode_string_field(3, &self.error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.command = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.working_directory = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShellPartialResult {
    pub stdout_delta: String,
    pub stderr_delta: String,
}

impl ShellPartialResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.stdout_delta));
        chunks.push(encode_string_field(2, &self.stderr_delta));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.stdout_delta = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.stderr_delta = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShellToolCall {
    pub args: Option<ShellArgs>,
    pub result: Option<ShellResult>,
}

impl ShellToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(ShellArgs::decode(&field.value)?);
                }
                2 => {
                    msg.result = Some(ShellResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShellToolCallStdoutDelta {
    pub content: String,
}

impl ShellToolCallStdoutDelta {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.content));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.content = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShellToolCallStderrDelta {
    pub content: String,
}

impl ShellToolCallStderrDelta {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.content));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.content = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShellToolCallDeltaDelta {
    Stdout(ShellToolCallStdoutDelta),
    Stderr(ShellToolCallStderrDelta),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShellToolCallDelta {
    pub delta: Option<ShellToolCallDeltaDelta>,
}

impl ShellToolCallDelta {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.delta {
            match val {
                ShellToolCallDeltaDelta::Stdout(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                ShellToolCallDeltaDelta::Stderr(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = ShellToolCallStdoutDelta::decode(&field.value)?;
                    msg.delta = Some(ShellToolCallDeltaDelta::Stdout(val));
                }
                2 => {
                    let val = ShellToolCallStderrDelta::decode(&field.value)?;
                    msg.delta = Some(ShellToolCallDeltaDelta::Stderr(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SubagentTypeType {
    Unspecified(SubagentTypeUnspecified),
    ComputerUse(SubagentTypeComputerUse),
    Custom(SubagentTypeCustom),
    Explore(SubagentTypeExplore),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SubagentType {
    pub type_: Option<SubagentTypeType>,
}

impl SubagentType {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.type_ {
            match val {
                SubagentTypeType::Unspecified(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                SubagentTypeType::ComputerUse(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                SubagentTypeType::Custom(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
                SubagentTypeType::Explore(ref inner) => {
                    chunks.push(encode_message_field(4, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = SubagentTypeUnspecified::decode(&field.value)?;
                    msg.type_ = Some(SubagentTypeType::Unspecified(val));
                }
                2 => {
                    let val = SubagentTypeComputerUse::decode(&field.value)?;
                    msg.type_ = Some(SubagentTypeType::ComputerUse(val));
                }
                3 => {
                    let val = SubagentTypeCustom::decode(&field.value)?;
                    msg.type_ = Some(SubagentTypeType::Custom(val));
                }
                4 => {
                    let val = SubagentTypeExplore::decode(&field.value)?;
                    msg.type_ = Some(SubagentTypeType::Explore(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SubagentTypeUnspecified {}

impl SubagentTypeUnspecified {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SubagentTypeComputerUse {}

impl SubagentTypeComputerUse {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SubagentTypeExplore {}

impl SubagentTypeExplore {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SubagentTypeCustom {
    pub name: String,
}

impl SubagentTypeCustom {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.name));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.name = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CustomSubagent {
    pub full_path: String,
    pub name: String,
    pub description: String,
    pub tools: Vec<String>,
    pub model: String,
    pub prompt: String,
    pub permission_mode: i32,
}

impl CustomSubagent {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.full_path));
        chunks.push(encode_string_field(2, &self.name));
        chunks.push(encode_string_field(3, &self.description));
        chunks.push(encode_repeated_string_field(4, &self.tools));
        chunks.push(encode_string_field(5, &self.model));
        chunks.push(encode_string_field(6, &self.prompt));
        if self.permission_mode != 0 {
            chunks.push(encode_varint_field_always(7, self.permission_mode as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.full_path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.name = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.description = String::from_utf8(field.value).ok()?;
                }
                4 => {
                    msg.tools.push(String::from_utf8(field.value).ok()?);
                }
                5 => {
                    msg.model = String::from_utf8(field.value).ok()?;
                }
                6 => {
                    msg.prompt = String::from_utf8(field.value).ok()?;
                }
                7 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.permission_mode = val as i32;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SwitchModeArgs {
    pub target_mode_id: String,
    pub explanation: Option<String>,
    pub tool_call_id: String,
}

impl SwitchModeArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.target_mode_id));
        if let Some(ref val) = self.explanation {
            chunks.push(encode_string_field_always(2, val));
        }
        chunks.push(encode_string_field(3, &self.tool_call_id));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.target_mode_id = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.explanation = Some(String::from_utf8(field.value).ok()?);
                }
                3 => {
                    msg.tool_call_id = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SwitchModeResultResult {
    Success(SwitchModeSuccess),
    Error(SwitchModeError),
    Rejected(SwitchModeRejected),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SwitchModeResult {
    pub result: Option<SwitchModeResultResult>,
}

impl SwitchModeResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                SwitchModeResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                SwitchModeResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                SwitchModeResultResult::Rejected(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = SwitchModeSuccess::decode(&field.value)?;
                    msg.result = Some(SwitchModeResultResult::Success(val));
                }
                2 => {
                    let val = SwitchModeError::decode(&field.value)?;
                    msg.result = Some(SwitchModeResultResult::Error(val));
                }
                3 => {
                    let val = SwitchModeRejected::decode(&field.value)?;
                    msg.result = Some(SwitchModeResultResult::Rejected(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SwitchModeSuccess {
    pub from_mode_id: String,
    pub to_mode_id: String,
}

impl SwitchModeSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.from_mode_id));
        chunks.push(encode_string_field(2, &self.to_mode_id));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.from_mode_id = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.to_mode_id = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SwitchModeError {
    pub error: String,
}

impl SwitchModeError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SwitchModeRejected {
    pub reason: String,
}

impl SwitchModeRejected {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.reason));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.reason = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SwitchModeToolCall {
    pub args: Option<SwitchModeArgs>,
    pub result: Option<SwitchModeResult>,
}

impl SwitchModeToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(SwitchModeArgs::decode(&field.value)?);
                }
                2 => {
                    msg.result = Some(SwitchModeResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SwitchModeRequestQuery {
    pub args: Option<SwitchModeArgs>,
}

impl SwitchModeRequestQuery {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(SwitchModeArgs::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SwitchModeRequestResponseResult {
    Approved(SwitchModeRequestResponse_Approved),
    Rejected(SwitchModeRequestResponse_Rejected),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SwitchModeRequestResponse {
    pub result: Option<SwitchModeRequestResponseResult>,
}

impl SwitchModeRequestResponse {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                SwitchModeRequestResponseResult::Approved(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                SwitchModeRequestResponseResult::Rejected(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = SwitchModeRequestResponse_Approved::decode(&field.value)?;
                    msg.result = Some(SwitchModeRequestResponseResult::Approved(val));
                }
                2 => {
                    let val = SwitchModeRequestResponse_Rejected::decode(&field.value)?;
                    msg.result = Some(SwitchModeRequestResponseResult::Rejected(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SwitchModeRequestResponse_Approved {}

impl SwitchModeRequestResponse_Approved {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SwitchModeRequestResponse_Rejected {
    pub reason: String,
}

impl SwitchModeRequestResponse_Rejected {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.reason));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.reason = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TodoItem {
    pub id: String,
    pub content: String,
    pub status: i32,
    pub created_at: i64,
    pub updated_at: i64,
    pub dependencies: Vec<String>,
}

impl TodoItem {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.id));
        chunks.push(encode_string_field(2, &self.content));
        if self.status != 0 {
            chunks.push(encode_varint_field_always(3, self.status as u64));
        }
        if self.created_at != 0 {
            chunks.push(encode_varint_field_always(4, self.created_at as u64));
        }
        if self.updated_at != 0 {
            chunks.push(encode_varint_field_always(5, self.updated_at as u64));
        }
        chunks.push(encode_repeated_string_field(6, &self.dependencies));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.id = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.content = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.status = val as i32;
                }
                4 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.created_at = val as i64;
                }
                5 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.updated_at = val as i64;
                }
                6 => {
                    msg.dependencies.push(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct UpdateTodosToolCall {
    pub args: Option<UpdateTodosArgs>,
    pub result: Option<UpdateTodosResult>,
}

impl UpdateTodosToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(UpdateTodosArgs::decode(&field.value)?);
                }
                2 => {
                    msg.result = Some(UpdateTodosResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct UpdateTodosArgs {
    pub todos: Vec<TodoItem>,
    pub merge: bool,
}

impl UpdateTodosArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        let items_todos: Vec<Vec<u8>> = self.todos.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(1, &items_todos));
        chunks.push(encode_bool_field(2, self.merge));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.todos.push(TodoItem::decode(&field.value)?);
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.merge = val != 0;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateTodosResultResult {
    Success(UpdateTodosSuccess),
    Error(UpdateTodosError),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct UpdateTodosResult {
    pub result: Option<UpdateTodosResultResult>,
}

impl UpdateTodosResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                UpdateTodosResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                UpdateTodosResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = UpdateTodosSuccess::decode(&field.value)?;
                    msg.result = Some(UpdateTodosResultResult::Success(val));
                }
                2 => {
                    let val = UpdateTodosError::decode(&field.value)?;
                    msg.result = Some(UpdateTodosResultResult::Error(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct UpdateTodosSuccess {
    pub todos: Vec<TodoItem>,
    pub total_count: i32,
    pub was_merge: bool,
}

impl UpdateTodosSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        let items_todos: Vec<Vec<u8>> = self.todos.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(1, &items_todos));
        if self.total_count != 0 {
            chunks.push(encode_varint_field_always(2, self.total_count as u64));
        }
        chunks.push(encode_bool_field(3, self.was_merge));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.todos.push(TodoItem::decode(&field.value)?);
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.total_count = val as i32;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.was_merge = val != 0;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct UpdateTodosError {
    pub error: String,
}

impl UpdateTodosError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadTodosToolCall {
    pub args: Option<ReadTodosArgs>,
    pub result: Option<ReadTodosResult>,
}

impl ReadTodosToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(ReadTodosArgs::decode(&field.value)?);
                }
                2 => {
                    msg.result = Some(ReadTodosResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadTodosArgs {
    pub status_filter: Vec<i32>,
    pub id_filter: Vec<String>,
}

impl ReadTodosArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_packed_varint_field(
            1,
            &self
                .status_filter
                .iter()
                .map(|&v| v as u64)
                .collect::<Vec<_>>(),
        ));
        chunks.push(encode_repeated_string_field(2, &self.id_filter));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    if field.wire_type == 2 {
                        let mut offset = 0;
                        while offset < field.value.len() {
                            let (val, len) = decode_varint(&field.value, offset)?;
                            msg.status_filter.push(val as i32);
                            offset += len;
                        }
                    } else {
                        let (val, _) = decode_varint(&field.value, 0)?;
                        msg.status_filter.push(val as i32);
                    }
                }
                2 => {
                    msg.id_filter.push(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReadTodosResultResult {
    Success(ReadTodosSuccess),
    Error(ReadTodosError),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadTodosResult {
    pub result: Option<ReadTodosResultResult>,
}

impl ReadTodosResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                ReadTodosResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                ReadTodosResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = ReadTodosSuccess::decode(&field.value)?;
                    msg.result = Some(ReadTodosResultResult::Success(val));
                }
                2 => {
                    let val = ReadTodosError::decode(&field.value)?;
                    msg.result = Some(ReadTodosResultResult::Error(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadTodosSuccess {
    pub todos: Vec<TodoItem>,
    pub total_count: i32,
}

impl ReadTodosSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        let items_todos: Vec<Vec<u8>> = self.todos.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(1, &items_todos));
        if self.total_count != 0 {
            chunks.push(encode_varint_field_always(2, self.total_count as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.todos.push(TodoItem::decode(&field.value)?);
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.total_count = val as i32;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadTodosError {
    pub error: String,
}

impl ReadTodosError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Range {
    pub start: Option<Position>,
    pub end: Option<Position>,
}

impl Range {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.start {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.end {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.start = Some(Position::decode(&field.value)?);
                }
                2 => {
                    msg.end = Some(Position::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}

impl Position {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.line != 0 {
            chunks.push(encode_varint_field_always(1, self.line as u64));
        }
        if self.column != 0 {
            chunks.push(encode_varint_field_always(2, self.column as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.line = val as u32;
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.column = val as u32;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Error {
    pub message: String,
}

impl Error {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.message));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.message = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WebSearchArgs {
    pub search_term: String,
    pub tool_call_id: String,
}

impl WebSearchArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.search_term));
        chunks.push(encode_string_field(2, &self.tool_call_id));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.search_term = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.tool_call_id = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WebSearchResultResult {
    Success(WebSearchSuccess),
    Error(WebSearchError),
    Rejected(WebSearchRejected),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WebSearchResult {
    pub result: Option<WebSearchResultResult>,
}

impl WebSearchResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                WebSearchResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                WebSearchResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                WebSearchResultResult::Rejected(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = WebSearchSuccess::decode(&field.value)?;
                    msg.result = Some(WebSearchResultResult::Success(val));
                }
                2 => {
                    let val = WebSearchError::decode(&field.value)?;
                    msg.result = Some(WebSearchResultResult::Error(val));
                }
                3 => {
                    let val = WebSearchRejected::decode(&field.value)?;
                    msg.result = Some(WebSearchResultResult::Rejected(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WebSearchSuccess {
    pub references: Vec<WebSearchReference>,
}

impl WebSearchSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        let items_references: Vec<Vec<u8>> =
            self.references.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(1, &items_references));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.references
                        .push(WebSearchReference::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WebSearchError {
    pub error: String,
}

impl WebSearchError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WebSearchRejected {
    pub reason: String,
}

impl WebSearchRejected {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.reason));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.reason = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WebSearchReference {
    pub title: String,
    pub url: String,
    pub chunk: String,
}

impl WebSearchReference {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.title));
        chunks.push(encode_string_field(2, &self.url));
        chunks.push(encode_string_field(3, &self.chunk));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.title = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.url = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.chunk = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WebSearchToolCall {
    pub args: Option<WebSearchArgs>,
    pub result: Option<WebSearchResult>,
}

impl WebSearchToolCall {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        if let Some(ref val) = self.result {
            chunks.push(encode_message_field(2, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(WebSearchArgs::decode(&field.value)?);
                }
                2 => {
                    msg.result = Some(WebSearchResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WebSearchRequestQuery {
    pub args: Option<WebSearchArgs>,
}

impl WebSearchRequestQuery {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.args {
            chunks.push(encode_message_field(1, &val.encode()));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.args = Some(WebSearchArgs::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WebSearchRequestResponseResult {
    Approved(WebSearchRequestResponse_Approved),
    Rejected(WebSearchRequestResponse_Rejected),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WebSearchRequestResponse {
    pub result: Option<WebSearchRequestResponseResult>,
}

impl WebSearchRequestResponse {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                WebSearchRequestResponseResult::Approved(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                WebSearchRequestResponseResult::Rejected(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = WebSearchRequestResponse_Approved::decode(&field.value)?;
                    msg.result = Some(WebSearchRequestResponseResult::Approved(val));
                }
                2 => {
                    let val = WebSearchRequestResponse_Rejected::decode(&field.value)?;
                    msg.result = Some(WebSearchRequestResponseResult::Rejected(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WebSearchRequestResponse_Approved {}

impl WebSearchRequestResponse_Approved {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WebSearchRequestResponse_Rejected {
    pub reason: String,
}

impl WebSearchRequestResponse_Rejected {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.reason));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.reason = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WriteArgs {
    pub path: String,
    pub file_text: String,
    pub tool_call_id: String,
    pub return_file_content_after_write: bool,
    pub file_bytes: Vec<u8>,
}

impl WriteArgs {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        chunks.push(encode_string_field(2, &self.file_text));
        chunks.push(encode_string_field(3, &self.tool_call_id));
        chunks.push(encode_bool_field(4, self.return_file_content_after_write));
        if !self.file_bytes.is_empty() {
            chunks.push(encode_message_field(5, &self.file_bytes));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.file_text = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.tool_call_id = String::from_utf8(field.value).ok()?;
                }
                4 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.return_file_content_after_write = val != 0;
                }
                5 => {
                    msg.file_bytes = field.value;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WriteResultResult {
    Success(WriteSuccess),
    PermissionDenied(WritePermissionDenied),
    NoSpace(WriteNoSpace),
    Error(WriteError),
    Rejected(WriteRejected),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WriteResult {
    pub result: Option<WriteResultResult>,
}

impl WriteResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.result {
            match val {
                WriteResultResult::Success(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                WriteResultResult::PermissionDenied(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
                WriteResultResult::NoSpace(ref inner) => {
                    chunks.push(encode_message_field(4, &inner.encode()));
                }
                WriteResultResult::Error(ref inner) => {
                    chunks.push(encode_message_field(5, &inner.encode()));
                }
                WriteResultResult::Rejected(ref inner) => {
                    chunks.push(encode_message_field(6, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = WriteSuccess::decode(&field.value)?;
                    msg.result = Some(WriteResultResult::Success(val));
                }
                3 => {
                    let val = WritePermissionDenied::decode(&field.value)?;
                    msg.result = Some(WriteResultResult::PermissionDenied(val));
                }
                4 => {
                    let val = WriteNoSpace::decode(&field.value)?;
                    msg.result = Some(WriteResultResult::NoSpace(val));
                }
                5 => {
                    let val = WriteError::decode(&field.value)?;
                    msg.result = Some(WriteResultResult::Error(val));
                }
                6 => {
                    let val = WriteRejected::decode(&field.value)?;
                    msg.result = Some(WriteResultResult::Rejected(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WriteSuccess {
    pub path: String,
    pub lines_created: i32,
    pub file_size: i32,
    pub file_content_after_write: Option<String>,
}

impl WriteSuccess {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        if self.lines_created != 0 {
            chunks.push(encode_varint_field_always(2, self.lines_created as u64));
        }
        if self.file_size != 0 {
            chunks.push(encode_varint_field_always(3, self.file_size as u64));
        }
        if let Some(ref val) = self.file_content_after_write {
            chunks.push(encode_string_field_always(4, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.lines_created = val as i32;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.file_size = val as i32;
                }
                4 => {
                    msg.file_content_after_write = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WritePermissionDenied {
    pub path: String,
    pub directory: String,
    pub operation: String,
    pub error: String,
    pub is_readonly: bool,
}

impl WritePermissionDenied {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        chunks.push(encode_string_field(2, &self.directory));
        chunks.push(encode_string_field(3, &self.operation));
        chunks.push(encode_string_field(4, &self.error));
        chunks.push(encode_bool_field(5, self.is_readonly));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.directory = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.operation = String::from_utf8(field.value).ok()?;
                }
                4 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                5 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.is_readonly = val != 0;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WriteNoSpace {
    pub path: String,
}

impl WriteNoSpace {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WriteError {
    pub path: String,
    pub error: String,
}

impl WriteError {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        chunks.push(encode_string_field(2, &self.error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WriteRejected {
    pub path: String,
    pub reason: String,
}

impl WriteRejected {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        chunks.push(encode_string_field(2, &self.reason));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.reason = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BootstrapStatsigRequest {
    pub ignore_dev_status: Option<bool>,
    pub operating_system: Option<i32>,
}

impl BootstrapStatsigRequest {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.ignore_dev_status {
            chunks.push(encode_bool_field_always(1, *val));
        }
        if let Some(ref val) = self.operating_system {
            chunks.push(encode_varint_field_always(2, *val as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.ignore_dev_status = Some(val != 0);
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.operating_system = Some(val as i32);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PingResponse {}

impl PingResponse {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExecRequest {
    pub command: String,
    pub cwd: Option<String>,
    pub args: Vec<String>,
    pub environment: std::collections::HashMap<String, String>,
}

impl ExecRequest {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.command));
        if let Some(ref val) = self.cwd {
            chunks.push(encode_string_field_always(2, val));
        }
        chunks.push(encode_repeated_string_field(3, &self.args));
        for (key, val) in &self.environment {
            let mut entry_chunks = Vec::new();
            entry_chunks.push(encode_string_field(1, key));
            entry_chunks.push(encode_string_field_always(2, val));
            chunks.push(encode_message_field(4, &concat_bytes(&entry_chunks)));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.command = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.cwd = Some(String::from_utf8(field.value).ok()?);
                }
                3 => {
                    msg.args.push(String::from_utf8(field.value).ok()?);
                }
                4 => {
                    let mut entry_key = String::new();
                    let mut entry_value = <String>::default();
                    for sub in parse_proto_fields(&field.value) {
                        match sub.number {
                            1 => entry_key = String::from_utf8(sub.value).ok()?,
                            2 => {
                                entry_value = String::from_utf8(sub.value).ok()?;
                            }
                            _ => {}
                        }
                    }
                    msg.environment.insert(entry_key, entry_value);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecResponseEvent {
    StdoutEvent(StdoutEvent),
    StderrEvent(StderrEvent),
    ExitEvent(ExitEvent),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExecResponse {
    pub event: Option<ExecResponseEvent>,
}

impl ExecResponse {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if let Some(ref val) = self.event {
            match val {
                ExecResponseEvent::StdoutEvent(ref inner) => {
                    chunks.push(encode_message_field(1, &inner.encode()));
                }
                ExecResponseEvent::StderrEvent(ref inner) => {
                    chunks.push(encode_message_field(2, &inner.encode()));
                }
                ExecResponseEvent::ExitEvent(ref inner) => {
                    chunks.push(encode_message_field(3, &inner.encode()));
                }
            }
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let val = StdoutEvent::decode(&field.value)?;
                    msg.event = Some(ExecResponseEvent::StdoutEvent(val));
                }
                2 => {
                    let val = StderrEvent::decode(&field.value)?;
                    msg.event = Some(ExecResponseEvent::StderrEvent(val));
                }
                3 => {
                    let val = ExitEvent::decode(&field.value)?;
                    msg.event = Some(ExecResponseEvent::ExitEvent(val));
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct StdoutEvent {
    pub data: String,
}

impl StdoutEvent {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.data));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.data = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct StderrEvent {
    pub data: String,
}

impl StderrEvent {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.data));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.data = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExitEvent {
    pub exit_code: i32,
}

impl ExitEvent {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.exit_code != 0 {
            chunks.push(encode_varint_field_always(1, self.exit_code as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.exit_code = val as i32;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadTextFileRequest {
    pub path: String,
}

impl ReadTextFileRequest {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadTextFileResponse {
    pub content: String,
}

impl ReadTextFileResponse {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.content));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.content = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WriteTextFileRequest {
    pub path: String,
    pub content: String,
}

impl WriteTextFileRequest {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        chunks.push(encode_string_field(2, &self.content));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.content = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WriteTextFileResponse {}

impl WriteTextFileResponse {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadBinaryFileRequest {
    pub path: String,
}

impl ReadBinaryFileRequest {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReadBinaryFileResponse {
    pub content: Vec<u8>,
}

impl ReadBinaryFileResponse {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if !self.content.is_empty() {
            chunks.push(encode_message_field(1, &self.content));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.content = field.value;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WriteBinaryFileRequest {
    pub path: String,
    pub content: Vec<u8>,
}

impl WriteBinaryFileRequest {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.path));
        if !self.content.is_empty() {
            chunks.push(encode_message_field(2, &self.content));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.content = field.value;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WriteBinaryFileResponse {}

impl WriteBinaryFileResponse {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GetWorkspaceChangesHashRequest {
    pub root_path: String,
    pub base_ref: String,
}

impl GetWorkspaceChangesHashRequest {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.root_path));
        chunks.push(encode_string_field(2, &self.base_ref));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.root_path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.base_ref = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GetWorkspaceChangesHashResponse {
    pub hash: String,
}

impl GetWorkspaceChangesHashResponse {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.hash));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.hash = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RefreshGithubAccessTokenRequest {
    pub github_access_token: String,
    pub hostname: String,
}

impl RefreshGithubAccessTokenRequest {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.github_access_token));
        chunks.push(encode_string_field(2, &self.hostname));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.github_access_token = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.hostname = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RefreshGithubAccessTokenResponse {}

impl RefreshGithubAccessTokenResponse {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WarmRemoteAccessServerRequest {
    pub commit: String,
    pub port: i32,
    pub connection_token: String,
}

impl WarmRemoteAccessServerRequest {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.commit));
        if self.port != 0 {
            chunks.push(encode_varint_field_always(2, self.port as u64));
        }
        chunks.push(encode_string_field(3, &self.connection_token));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.commit = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.port = val as i32;
                }
                3 => {
                    msg.connection_token = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WarmRemoteAccessServerResponse {}

impl WarmRemoteAccessServerResponse {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ListArtifactsRequest {}

impl ListArtifactsRequest {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ArtifactUploadMetadata {
    pub absolute_path: String,
    pub size_bytes: u64,
    pub updated_at_unix_ms: i64,
    pub status: i32,
    pub bytes_uploaded: u64,
    pub last_error: String,
    pub upload_attempts: u32,
    pub last_started_at_unix_ms: i64,
    pub last_finished_at_unix_ms: i64,
    pub upload_id: String,
}

impl ArtifactUploadMetadata {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.absolute_path));
        if self.size_bytes != 0 {
            chunks.push(encode_varint_field_always(2, self.size_bytes as u64));
        }
        if self.updated_at_unix_ms != 0 {
            chunks.push(encode_varint_field_always(
                3,
                self.updated_at_unix_ms as u64,
            ));
        }
        if self.status != 0 {
            chunks.push(encode_varint_field_always(4, self.status as u64));
        }
        if self.bytes_uploaded != 0 {
            chunks.push(encode_varint_field_always(5, self.bytes_uploaded as u64));
        }
        chunks.push(encode_string_field(6, &self.last_error));
        if self.upload_attempts != 0 {
            chunks.push(encode_varint_field_always(7, self.upload_attempts as u64));
        }
        if self.last_started_at_unix_ms != 0 {
            chunks.push(encode_varint_field_always(
                8,
                self.last_started_at_unix_ms as u64,
            ));
        }
        if self.last_finished_at_unix_ms != 0 {
            chunks.push(encode_varint_field_always(
                9,
                self.last_finished_at_unix_ms as u64,
            ));
        }
        chunks.push(encode_string_field(10, &self.upload_id));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.absolute_path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.size_bytes = val as u64;
                }
                3 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.updated_at_unix_ms = val as i64;
                }
                4 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.status = val as i32;
                }
                5 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.bytes_uploaded = val as u64;
                }
                6 => {
                    msg.last_error = String::from_utf8(field.value).ok()?;
                }
                7 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.upload_attempts = val as u32;
                }
                8 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.last_started_at_unix_ms = val as i64;
                }
                9 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.last_finished_at_unix_ms = val as i64;
                }
                10 => {
                    msg.upload_id = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ListArtifactsResponse {
    pub artifacts: Vec<ArtifactUploadMetadata>,
}

impl ListArtifactsResponse {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        let items_artifacts: Vec<Vec<u8>> =
            self.artifacts.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(1, &items_artifacts));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.artifacts
                        .push(ArtifactUploadMetadata::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct UploadArtifactsRequest {
    pub uploads: Vec<ArtifactUploadInstruction>,
}

impl UploadArtifactsRequest {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        let items_uploads: Vec<Vec<u8>> = self.uploads.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(1, &items_uploads));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.uploads
                        .push(ArtifactUploadInstruction::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ArtifactUploadInstruction {
    pub absolute_path: String,
    pub upload_url: String,
    pub method: String,
    pub headers: std::collections::HashMap<String, String>,
    pub content_type: Option<String>,
    pub slack_upload_url: Option<String>,
    pub slack_file_id: Option<String>,
}

impl ArtifactUploadInstruction {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.absolute_path));
        chunks.push(encode_string_field(2, &self.upload_url));
        chunks.push(encode_string_field(3, &self.method));
        for (key, val) in &self.headers {
            let mut entry_chunks = Vec::new();
            entry_chunks.push(encode_string_field(1, key));
            entry_chunks.push(encode_string_field_always(2, val));
            chunks.push(encode_message_field(4, &concat_bytes(&entry_chunks)));
        }
        if let Some(ref val) = self.content_type {
            chunks.push(encode_string_field_always(5, val));
        }
        if let Some(ref val) = self.slack_upload_url {
            chunks.push(encode_string_field_always(6, val));
        }
        if let Some(ref val) = self.slack_file_id {
            chunks.push(encode_string_field_always(7, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.absolute_path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.upload_url = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.method = String::from_utf8(field.value).ok()?;
                }
                4 => {
                    let mut entry_key = String::new();
                    let mut entry_value = <String>::default();
                    for sub in parse_proto_fields(&field.value) {
                        match sub.number {
                            1 => entry_key = String::from_utf8(sub.value).ok()?,
                            2 => {
                                entry_value = String::from_utf8(sub.value).ok()?;
                            }
                            _ => {}
                        }
                    }
                    msg.headers.insert(entry_key, entry_value);
                }
                5 => {
                    msg.content_type = Some(String::from_utf8(field.value).ok()?);
                }
                6 => {
                    msg.slack_upload_url = Some(String::from_utf8(field.value).ok()?);
                }
                7 => {
                    msg.slack_file_id = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ArtifactUploadDispatchResult {
    pub absolute_path: String,
    pub status: i32,
    pub message: String,
    pub slack_file_id: Option<String>,
}

impl ArtifactUploadDispatchResult {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.absolute_path));
        if self.status != 0 {
            chunks.push(encode_varint_field_always(2, self.status as u64));
        }
        chunks.push(encode_string_field(3, &self.message));
        if let Some(ref val) = self.slack_file_id {
            chunks.push(encode_string_field_always(4, val));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.absolute_path = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.status = val as i32;
                }
                3 => {
                    msg.message = String::from_utf8(field.value).ok()?;
                }
                4 => {
                    msg.slack_file_id = Some(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct UploadArtifactsResponse {
    pub results: Vec<ArtifactUploadDispatchResult>,
}

impl UploadArtifactsResponse {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        let items_results: Vec<Vec<u8>> = self.results.iter().map(|item| item.encode()).collect();
        chunks.push(encode_repeated_message_field(1, &items_results));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.results
                        .push(ArtifactUploadDispatchResult::decode(&field.value)?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GetMcpRefreshTokensRequest {}

impl GetMcpRefreshTokensRequest {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GetMcpRefreshTokensResponse {
    pub refresh_tokens: std::collections::HashMap<String, String>,
}

impl GetMcpRefreshTokensResponse {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        for (key, val) in &self.refresh_tokens {
            let mut entry_chunks = Vec::new();
            entry_chunks.push(encode_string_field(1, key));
            entry_chunks.push(encode_string_field_always(2, val));
            chunks.push(encode_message_field(1, &concat_bytes(&entry_chunks)));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let mut entry_key = String::new();
                    let mut entry_value = <String>::default();
                    for sub in parse_proto_fields(&field.value) {
                        match sub.number {
                            1 => entry_key = String::from_utf8(sub.value).ok()?,
                            2 => {
                                entry_value = String::from_utf8(sub.value).ok()?;
                            }
                            _ => {}
                        }
                    }
                    msg.refresh_tokens.insert(entry_key, entry_value);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct UpdateEnvironmentVariablesRequest {
    pub env: std::collections::HashMap<String, String>,
    pub replace: bool,
}

impl UpdateEnvironmentVariablesRequest {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        for (key, val) in &self.env {
            let mut entry_chunks = Vec::new();
            entry_chunks.push(encode_string_field(1, key));
            entry_chunks.push(encode_string_field_always(2, val));
            chunks.push(encode_message_field(1, &concat_bytes(&entry_chunks)));
        }
        chunks.push(encode_bool_field(2, self.replace));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let mut entry_key = String::new();
                    let mut entry_value = <String>::default();
                    for sub in parse_proto_fields(&field.value) {
                        match sub.number {
                            1 => entry_key = String::from_utf8(sub.value).ok()?,
                            2 => {
                                entry_value = String::from_utf8(sub.value).ok()?;
                            }
                            _ => {}
                        }
                    }
                    msg.env.insert(entry_key, entry_value);
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.replace = val != 0;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct UpdateEnvironmentVariablesResponse {
    pub applied: u32,
    pub removed: u32,
}

impl UpdateEnvironmentVariablesResponse {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        if self.applied != 0 {
            chunks.push(encode_varint_field_always(1, self.applied as u64));
        }
        if self.removed != 0 {
            chunks.push(encode_varint_field_always(2, self.removed as u64));
        }
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.applied = val as u32;
                }
                2 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.removed = val as u32;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct McpOAuthStoredData {
    pub refresh_token: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub redirect_uris: Vec<String>,
}

impl McpOAuthStoredData {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.refresh_token));
        chunks.push(encode_string_field(2, &self.client_id));
        if let Some(ref val) = self.client_secret {
            chunks.push(encode_string_field_always(3, val));
        }
        chunks.push(encode_repeated_string_field(4, &self.redirect_uris));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.refresh_token = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.client_id = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.client_secret = Some(String::from_utf8(field.value).ok()?);
                }
                4 => {
                    msg.redirect_uris.push(String::from_utf8(field.value).ok()?);
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Frame {
    pub id: String,
    pub method: String,
    pub data: Vec<u8>,
    pub kind: i32,
    pub error: String,
}

impl Frame {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.id));
        chunks.push(encode_string_field(2, &self.method));
        if !self.data.is_empty() {
            chunks.push(encode_message_field(3, &self.data));
        }
        if self.kind != 0 {
            chunks.push(encode_varint_field_always(4, self.kind as u64));
        }
        chunks.push(encode_string_field(5, &self.error));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.id = String::from_utf8(field.value).ok()?;
                }
                2 => {
                    msg.method = String::from_utf8(field.value).ok()?;
                }
                3 => {
                    msg.data = field.value;
                }
                4 => {
                    let (val, _) = decode_varint(&field.value, 0)?;
                    msg.kind = val as i32;
                }
                5 => {
                    msg.error = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Empty {}

impl Empty {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BidiRequestId {
    pub request_id: String,
}

impl BidiRequestId {
    pub fn encode(&self) -> Vec<u8> {
        let mut chunks = Vec::new();
        chunks.push(encode_string_field(1, &self.request_id));
        concat_bytes(&chunks)
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut msg = Self::default();
        for field in parse_proto_fields(data) {
            match field.number {
                1 => {
                    msg.request_id = String::from_utf8(field.value).ok()?;
                }
                _ => {} // Ignore unknown fields
            }
        }
        Some(msg)
    }
}
