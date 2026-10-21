//! 词库导入完成后的收尾。

use super::Settings;
use crate::jobs::import::Report;
use crate::library::{Kind, Library};

impl Settings {
    /// 导入完了：之前关掉过的同名词库重新打开（码表改成用刚导入的那张），重扫清单，把结果显示出来。
    pub(super) fn imported(&mut self, report: Report) {
        let kind = report.kind;
        let (section, key) = kind.disabled_key();
        let disabled = match kind {
            Kind::Dictionary => &self.config.dictionaries.disabled,
            Kind::CodeTable => &self.config.aux_code.disabled,
        };
        let kept: Vec<String> = disabled
            .iter()
            .filter(|stem| !report.imported.contains(stem))
            .cloned()
            .collect();
        if kept.len() != disabled.len() {
            self.write_list(section, key, &kept);
        }
        if kind == Kind::CodeTable
            && let Some(stem) = report.imported.last()
        {
            self.write("aux_code", "scheme", stem.as_str());
        }
        self.library = Library::scan();
        self.notice = Some((kind.slot(), report.lines.join("\n")));
    }
}
