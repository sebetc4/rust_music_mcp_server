//! Tool Router - builds the rmcp ToolRouter from registry.
//!
//! This module builds the ToolRouter for STDIO/TCP transport by delegating
//! to the tool definitions themselves. Each tool knows how to create its own route.

use std::sync::Arc;

use rmcp::handler::server::tool::ToolRouter;

use crate::core::config::Config;
use crate::domains::tools::definitions::MbIdentifyRecordTool;

use super::definitions::{
    FsDeleteTool, FsListDirTool, FsRenameTool, MbArtistTool, MbCoverDownloadTool, MbLabelTool,
    MbRecordingTool, MbReleaseTool, MbWorkTool, ReadMetadataTool, WriteMetadataTool,
};

/// Build the tool router with all registered tools.
pub fn build_tool_router<S>(config: Arc<Config>) -> ToolRouter<S>
where
    S: Send + Sync + 'static,
{
    ToolRouter::new()
        .with_route(FsDeleteTool::create_route(config.clone()))
        .with_route(FsListDirTool::create_route(config.clone()))
        .with_route(FsRenameTool::create_route(config.clone()))
        .with_route(MbArtistTool::create_route())
        .with_route(MbCoverDownloadTool::create_route(config.clone()))
        .with_route(MbIdentifyRecordTool::create_route(config.clone()))
        .with_route(MbLabelTool::create_route())
        .with_route(MbRecordingTool::create_route())
        .with_route(MbReleaseTool::create_route())
        .with_route(MbWorkTool::create_route())
        .with_route(ReadMetadataTool::create_route(config.clone()))
        .with_route(WriteMetadataTool::create_route(config))
}

#[cfg(test)]
mod tests {
    use super::super::registry::ToolRegistry;
    use super::*;

    struct TestServer {}

    fn test_config() -> Arc<Config> {
        Arc::new(Config::default())
    }

    #[test]
    fn test_build_router() {
        let router: ToolRouter<TestServer> = build_tool_router(test_config());
        let tools = router.list_all();
        assert_eq!(tools.len(), 12);

        let names: Vec<_> = tools.iter().map(|t| t.name.as_ref()).collect();
        assert!(names.contains(&"fs_delete"));
        assert!(names.contains(&"fs_list_dir"));
        assert!(names.contains(&"mb_artist_search"));
        assert!(names.contains(&"mb_cover_download"));
        assert!(names.contains(&"mb_release_search"));
        assert!(names.contains(&"mb_recording_search"));
        assert!(names.contains(&"mb_label_search"));
        assert!(names.contains(&"mb_work_search"));
        assert!(names.contains(&"mb_identify_record"));
    }

    #[test]
    fn test_registry_matches_router() {
        // Ensure registry and router have the same tools
        let config = test_config();
        let registry = ToolRegistry::new(config.clone());
        let registry_names = registry.tool_names();

        let router: ToolRouter<TestServer> = build_tool_router(config);
        let router_tools = router.list_all();
        let router_names: Vec<_> = router_tools.iter().map(|t| t.name.as_ref()).collect();

        assert_eq!(registry_names.len(), router_names.len());
        for name in registry_names {
            assert!(router_names.contains(&name));
        }
    }
}
