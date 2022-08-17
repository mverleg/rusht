
pub struct KeyBuilder<'a> {
    template: String,
    generators: Vec<(&'a str, &'a dyn FnOnce() -> String)>,
}

impl <'a> KeyBuilder<'a> {
    pub fn new(template: impl Into<String>) -> Self {
        KeyBuilder {
            template: template.into(),
            generators: vec![],
        }
    }

    pub fn add(&mut self, name: &'a str, generator: &'a dyn FnOnce() -> String) {
        self.generators.push((name, generator));
    }

    pub fn build(self) -> String {
        let mut key = self.template;
        for (name, generator) in self.generators {
            let token = format!("${{{}}}", name);
            if key.contains(&token) {
                let value = generator();
                key.replace(&token, &value);
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
        let fn_once: dyn FnOnce() -> String = &|| owned_home_value;
        builder.add("pwd", &|| "hello/world".to_owned());
        builder.add("env", &fn_once);
        builder.add("cmd", &|| panic!());
        let key = builder.build();
        assert_eq!(key, "");
    }
}
