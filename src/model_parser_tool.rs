#![allow(dead_code)]
use model_parser_mcp::model::{
    app_state::{self, AppState},
    cubs_model::ModelVersionNumber,
    model_parser::ModelParser,
};
use rmcp::{
    ServerHandler,
    handler::server::{
        router::tool::ToolRouter,
        wrapper::{Json, Parameters},
    },
    model::{CallToolResult, ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// https://github.com/modelcontextprotocol/rust-sdk/blob/main/crates/rmcp/README.md
// https://hackmd.io/@Hamze/S1tlKZP0kx

#[derive(Clone)]
pub struct ModelParserTool {
    tool_router: ToolRouter<Self>,
    app_state: AppState,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelStatsResult {
    pub model_id: String,
    pub stats: String,
    pub types: Vec<String>,
    pub natures: Vec<String>,
    pub current_version: String,
    pub all_model_versions: Vec<ModelVersionNumber>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelStatsErrorResult {
    pub model_id: String,
    pub error_msg: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ModelInfoRequest {
    #[schemars(description = "Unique identifier for a model in the format of UUID")]
    model_id: String,
    #[schemars(description = " Model version")]
    version_number: Option<String>,
}

#[tool_router]
impl ModelParserTool {
    pub fn new(app_state: AppState) -> Self {
        // let app_state = AppState::global();
        Self {
            tool_router: Self::tool_router(),
            app_state: app_state,
        }
    }

    #[tool(description = "Get model infomation using model model id and model version")]
    async fn get_model_stats(
        &self,
        Parameters(ModelInfoRequest {
            model_id,
            version_number,
        }): Parameters<ModelInfoRequest>,
    ) -> String {
        let model_parser = ModelParser::new(
            self.app_state.get_model_cache(),
            self.app_state.get_graph_cache(),
            self.app_state.get_pg_pool_ref(),
        );
        let version_number = version_number.unwrap_or("".to_string());

        let model_dict = model_parser
            .get_model_stats(&model_id, &version_number)
            .await;

        match model_dict {
            Ok(dict) => {
                let model_stats = serde_json::to_string_pretty(&dict.model_stats).unwrap();
                let result = ModelStatsResult {
                    model_id: model_id,
                    stats: model_stats,
                    types: dict.get_element_types(),
                    natures: dict.get_element_nature(),
                    current_version: dict.version.to_string(),
                    all_model_versions: dict.model_versions,
                };
                serde_json::to_string_pretty(&result).unwrap()
            }
            Err(e) => {
                let error = ModelStatsErrorResult {
                    model_id,
                    error_msg: e.to_string(),
                };
                serde_json::to_string_pretty(&error).unwrap()
            }
        }
    }
}

// Implement the server handler
#[tool_handler]
impl ServerHandler for ModelParserTool {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "A simple parser that retrieve information regarding the model.".into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
