use crate::AppInfo;

pub struct Deposit {
    content: Vec<AppInfo>
}
impl Deposit {
    pub fn from(content: Vec<AppInfo>) -> Deposit {
        Deposit {
            content
        }
    }

    pub fn push(&mut self, object: AppInfo) {
        self.content.push(object);
    }

    pub fn search(&self, name: &str) -> Vec<AppInfo> {
        let mut found: Vec<AppInfo> = Vec::new();
        for (index, app_info) in self.content.iter().enumerate() {
            if app_info.name.contains(name) {
                found.push(app_info.clone());
            }
        }
        found
    }

    pub fn get_all(&self) -> Vec<AppInfo> {
        self.content.clone()
    }

    pub fn replace(&mut self, new_content: Vec<AppInfo>) {
        self.content = new_content
    }

}