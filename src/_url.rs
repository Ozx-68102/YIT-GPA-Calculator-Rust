use url::Url;

pub const HTTP_SCHEME: &str = "http";
const HOST: &str = "jw.yit.edu.cn";

pub struct AAOUrl {
    url: Url,
}

impl AAOUrl {
    pub fn new() -> Self {
        let mut url = Url::parse(&format!("{}://{}/", HTTP_SCHEME, HOST)).unwrap();
        url.path_segments_mut().unwrap()
            .push("yjlgxy_jsxsd").push("");

        Self { url }
    }

    /// 丝滑转换为 Url 类型
    pub fn get(&self) -> Url {
        self.url.clone()
    }

    /// 适配 base_url 的末尾斜杠，改写 push 的逻辑。
    /// 不改变原来的 base 实例, 返回一个新的实例且支持链式调用
    pub fn push(&self, segment: &str) -> Self {
        let mut url = self.url.clone();
        url.path_segments_mut().unwrap()
            .pop_if_empty().push(segment);

        Self { url }
    }
}
