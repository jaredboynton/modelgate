// Generated automatically from agent_pb.ts schema. Do not edit manually.

pub mod agent_service {
    pub const PATH_RUN: &str = "/agent.AgentService/Run";
    pub const PATH_RUN_S_S_E: &str = "/agent.AgentService/RunSSE";
    pub const PATH_NAME_AGENT: &str = "/agent.AgentService/NameAgent";
    pub const PATH_GET_USABLE_MODELS: &str = "/agent.AgentService/GetUsableModels";
    pub const PATH_GET_DEFAULT_MODEL_FOR_CLI: &str = "/agent.AgentService/GetDefaultModelForCli";
    pub const PATH_GET_ALLOWED_MODEL_INTENTS: &str = "/agent.AgentService/GetAllowedModelIntents";
}

pub mod control_service {
    pub const PATH_READ_TEXT_FILE: &str = "/agent.ControlService/ReadTextFile";
    pub const PATH_WRITE_TEXT_FILE: &str = "/agent.ControlService/WriteTextFile";
    pub const PATH_READ_BINARY_FILE: &str = "/agent.ControlService/ReadBinaryFile";
    pub const PATH_WRITE_BINARY_FILE: &str = "/agent.ControlService/WriteBinaryFile";
    pub const PATH_GET_WORKSPACE_CHANGES_HASH: &str =
        "/agent.ControlService/GetWorkspaceChangesHash";
    pub const PATH_REFRESH_GITHUB_ACCESS_TOKEN: &str =
        "/agent.ControlService/RefreshGithubAccessToken";
    pub const PATH_WARM_REMOTE_ACCESS_SERVER: &str = "/agent.ControlService/WarmRemoteAccessServer";
    pub const PATH_LIST_ARTIFACTS: &str = "/agent.ControlService/ListArtifacts";
    pub const PATH_UPLOAD_ARTIFACTS: &str = "/agent.ControlService/UploadArtifacts";
    pub const PATH_GET_MCP_REFRESH_TOKENS: &str = "/agent.ControlService/GetMcpRefreshTokens";
    pub const PATH_UPDATE_ENVIRONMENT_VARIABLES: &str =
        "/agent.ControlService/UpdateEnvironmentVariables";
}

pub mod exec_service {}

pub mod privateworkerbridgeexternal_service {
    pub const PATH_CONNECT: &str = "/agent.PrivateWorkerBridgeExternalService/Connect";
}

pub mod lifecycle_service {
    pub const PATH_RESET_INSTANCE: &str = "/agent.LifecycleService/ResetInstance";
    pub const PATH_RENEW_INSTANCE: &str = "/agent.LifecycleService/RenewInstance";
}
