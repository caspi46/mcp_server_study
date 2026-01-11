use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::schemars::JsonSchema;
use rmcp::{ErrorData as McpError, ServerHandler, tool, tool_handler, tool_router};
use serde::Deserialize;
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BinaryOpArgs {
    pub a: f64,
    pub b: f64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RepoOverviewArgs {
    pub repo_root: String,
    pub max_depth: Option<usize>,
    pub include_hidden: Option<bool>,
}
// #[derive(Debug, Deserialize, JsonSchema)]
// pub struct Repo {
//     pub repo_root: string,
//     pub max_depth: Optional<i32>,
// }

#[derive(Debug, Clone, serde::Serialize)]
pub struct RepoOverview {
    pub name: String,
    pub total_files: usize,
    pub total_dirs: usize,
    pub languages: HashMap<String, usize>, // raw counts
    pub has_git: bool,
}

pub fn compute_repo_overview(
    repo_root: &PathBuf,
    max_depth: usize,
    include_hidden: bool,
) -> Result<RepoOverview, std::io::Error> {
    let mut total_files = 0;
    let mut total_dirs = 0;
    let mut languages: HashMap<String, usize> = HashMap::new();
    let has_git = repo_root.join(".git").exists();

    let wd_max_depth = max_depth.saturating_add(1);

    let walker = WalkDir::new(repo_root)
        .max_depth(wd_max_depth)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            if include_hidden {
                true
            } else {
                if e.depth() == 0 {
                    true
                } else {
                    e.file_name()
                        .to_str()
                        .map(|s| !s.starts_with('.'))
                        .unwrap_or(false)
                }
            }
        });

    // Walk through the directory
    for entry in walker {
        let entry = entry?;
        let path = entry.path();

        if entry.depth() == 0 {
            continue;
        }

        if path.is_dir() {
            total_dirs += 1;
        } else if path.is_file() {
            total_files += 1;
            // Count by file extension
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let lang = match ext {
                    "rs" => "Rust",
                    "java" => "Java",
                    "cpp" => "C++",
                    "c" => "C",
                    "ts" => "TypeScript",
                    "js" => "JavaScript",
                    "py" => "Python",
                    "md" => "Markdown",
                    "toml" => "TOML",
                    "json" => "JSON",
                    _ => "Others",
                };
                *languages.entry(lang.to_string()).or_insert(0) += 1;
            } else {
                *languages.entry("NoExt".to_string()).or_insert(0) += 1;
            }
        }
    }

    let name = repo_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(RepoOverview {
        name,
        total_files,
        total_dirs,
        languages,
        has_git,
    })
}

#[derive(Clone)]
pub struct RepoServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl RepoServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Add two numbers")]
    pub async fn add(
        &self,
        Parameters(args): Parameters<BinaryOpArgs>,
    ) -> Result<CallToolResult, McpError> {
        let result = args.a + args.b;
        Ok(CallToolResult::success(vec![Content::text(
            result.to_string(),
        )]))
    }

    #[tool(description = "shows the repo overview structure")]
    pub async fn overview_structure(
        &self,
        Parameters(args): Parameters<RepoOverviewArgs>,
    ) -> Result<CallToolResult, McpError> {
        let repo_root = PathBuf::from(&args.repo_root);
        if !repo_root.exists() || !repo_root.is_dir() {
            return Err(McpError::invalid_params("invalid params", None));
        }

        let overview = compute_repo_overview(
            &repo_root,
            args.max_depth.unwrap_or(usize::MAX),
            args.include_hidden.unwrap_or(false),
        )
        .map_err(|e| McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: Cow::Owned(format!("repo_overview failed: {e}")),
            data: None,
        })?;

        let json_text = serde_json::to_string_pretty(&overview).map_err(|e| McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: Cow::Owned(format!("repo_overview failed: {e}")),
            data: None,
        })?;

        Ok(CallToolResult::success(vec![Content::text(json_text)]))
    }
}
#[tool_handler]
impl ServerHandler for RepoServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("Basic calculator: add, subtract, multiply, divide".to_string()),
        }
    }
}
