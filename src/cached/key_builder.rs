
pub struct KeyBuilder {
    template: String,
    generators: Vec<(String, Box<dyn FnOnce() -> String>)>,
}

impl KeyBuilder {
    pub fn new(template: impl Into<String>) -> Self {
        KeyBuilder {
            template: template.into(),
            generators: vec![],
        }
    }

    pub fn add(&mut self, name: impl Into<String>, generator: Box<dyn FnOnce() -> String>) {
        self.generators.push((name.into(), generator));
    }

    pub fn build(self) -> String {
        let mut key = self.template;
        for (name, generator) in self.generators {
            let token = format!("${{{}}}", name);
            if key.contains(&token) {
                let value = generator();
                key = key.replace(&token, &value);
            }
        }
        debug_assert!(!key.contains("${"));
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_key() {
        let mut builder = KeyBuilder::new("${pwd}_${env}.cache");
        let owned_home_value = "HOME=/Users/me".to_owned();
        let fn_once: Box<dyn FnOnce() -> String> = Box::new(|| owned_home_value);
        builder.add("pwd", Box::new(|| "hello/world".to_owned()));
        builder.add("env", fn_once);
        builder.add("cmd", Box::new(|| panic!()));
        let key = builder.build();
        assert_eq!(key, "hello/world_HOME=/Users/me.cache");
    }
}
