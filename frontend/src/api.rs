use gloo_net::http::Request;

use crate::types::{AssetNode, AssetsResponse, DirectoriesResponse, DirectoryNode};

/// 获取根目录列表
pub async fn get_root_directories() -> Result<Vec<DirectoryNode>, String> {
    match Request::get("/api/nodes/root").send().await {
        Ok(resp) => match resp.json::<DirectoriesResponse>().await {
            Ok(data) => Ok(data.directories),
            Err(e) => Err(format!("解析错误: {e}")),
        },
        Err(e) => Err(format!("请求失败: {e}")),
    }
}

/// 获取子目录列表
pub async fn get_child_directories(path: &str) -> Result<Vec<DirectoryNode>, String> {
    let encoded_path = urlencoding::encode(path);
    let url = format!("/api/nodes/children/{}", encoded_path);
    match Request::get(&url).send().await {
        Ok(resp) => match resp.json::<DirectoriesResponse>().await {
            Ok(data) => Ok(data.directories),
            Err(e) => Err(format!("解析错误: {e}")),
        },
        Err(e) => Err(format!("请求失败: {e}")),
    }
}

/// 获取节点资源文件列表
pub async fn get_node_assets(path: &str) -> Result<Vec<AssetNode>, String> {
    let encoded_path = urlencoding::encode(path);
    let url = format!("/api/nodes/assets/{}", encoded_path);
    match Request::get(&url).send().await {
        Ok(resp) => match resp.json::<AssetsResponse>().await {
            Ok(data) => Ok(data.assets),
            Err(e) => Err(format!("解析错误: {e}")),
        },
        Err(e) => Err(format!("请求失败: {e}")),
    }
}
