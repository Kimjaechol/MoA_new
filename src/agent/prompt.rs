use crate::config::IdentityConfig;
use crate::identity;
use crate::skills::Skill;
use crate::tools::Tool;
use anyhow::Result;
use chrono::Local;
use std::fmt::Write;
use std::path::Path;

const BOOTSTRAP_MAX_CHARS: usize = 20_000;

pub struct PromptContext<'a> {
    pub workspace_dir: &'a Path,
    pub model_name: &'a str,
    pub tools: &'a [Box<dyn Tool>],
    pub skills: &'a [Skill],
    pub identity_config: Option<&'a IdentityConfig>,
    pub dispatcher_instructions: &'a str,
}

pub trait PromptSection: Send + Sync {
    fn name(&self) -> &str;
    fn build(&self, ctx: &PromptContext<'_>) -> Result<String>;
}

#[derive(Default)]
pub struct SystemPromptBuilder {
    sections: Vec<Box<dyn PromptSection>>,
}

impl SystemPromptBuilder {
    pub fn with_defaults() -> Self {
        Self {
            sections: vec![
                Box::new(IdentitySection),
                Box::new(ToolsSection),
                Box::new(SafetySection),
                Box::new(SchedulingSection),
                Box::new(SkillsSection),
                Box::new(WorkspaceSection),
                Box::new(DateTimeSection),
                Box::new(RuntimeSection),
            ],
        }
    }

    pub fn add_section(mut self, section: Box<dyn PromptSection>) -> Self {
        self.sections.push(section);
        self
    }

    pub fn build(&self, ctx: &PromptContext<'_>) -> Result<String> {
        let mut output = String::new();
        for section in &self.sections {
            let part = section.build(ctx)?;
            if part.trim().is_empty() {
                continue;
            }
            output.push_str(part.trim_end());
            output.push_str("\n\n");
        }
        Ok(output)
    }
}

pub struct IdentitySection;
pub struct ToolsSection;
pub struct SafetySection;
pub struct SchedulingSection;
pub struct SkillsSection;
pub struct WorkspaceSection;
pub struct RuntimeSection;
pub struct DateTimeSection;

impl PromptSection for IdentitySection {
    fn name(&self) -> &str {
        "identity"
    }

    fn build(&self, ctx: &PromptContext<'_>) -> Result<String> {
        let mut prompt = String::from("## Project Context\n\n");
        if let Some(config) = ctx.identity_config {
            if identity::is_aieos_configured(config) {
                if let Ok(Some(aieos)) = identity::load_aieos_identity(config, ctx.workspace_dir) {
                    let rendered = identity::aieos_to_system_prompt(&aieos);
                    if !rendered.is_empty() {
                        prompt.push_str(&rendered);
                        return Ok(prompt);
                    }
                }
            }
        }

        prompt.push_str(
            "The following workspace files define your identity, behavior, and context.\n\n",
        );
        for file in [
            "AGENTS.md",
            "SOUL.md",
            "TOOLS.md",
            "IDENTITY.md",
            "USER.md",
            "HEARTBEAT.md",
            "BOOTSTRAP.md",
            "MEMORY.md",
        ] {
            inject_workspace_file(&mut prompt, ctx.workspace_dir, file);
        }

        Ok(prompt)
    }
}

impl PromptSection for ToolsSection {
    fn name(&self) -> &str {
        "tools"
    }

    fn build(&self, ctx: &PromptContext<'_>) -> Result<String> {
        let mut out = String::from("## Tools\n\n");
        for tool in ctx.tools {
            let _ = writeln!(
                out,
                "- **{}**: {}\n  Parameters: `{}`",
                tool.name(),
                tool.description(),
                tool.parameters_schema()
            );
        }
        if !ctx.dispatcher_instructions.is_empty() {
            out.push('\n');
            out.push_str(ctx.dispatcher_instructions);
        }
        Ok(out)
    }
}

impl PromptSection for SafetySection {
    fn name(&self) -> &str {
        "safety"
    }

    fn build(&self, _ctx: &PromptContext<'_>) -> Result<String> {
        Ok("## Safety\n\n- Do not exfiltrate private data.\n- Do not run destructive commands without asking.\n- Do not bypass oversight or approval mechanisms.\n- Prefer `trash` over `rm`.\n- When in doubt, ask before acting externally.".into())
    }
}

impl PromptSection for SchedulingSection {
    fn name(&self) -> &str {
        "scheduling"
    }

    fn build(&self, ctx: &PromptContext<'_>) -> Result<String> {
        let has_cron = ctx.tools.iter().any(|t| t.name() == "cron_add");
        if !has_cron {
            return Ok(String::new());
        }

        let has_browser = ctx.tools.iter().any(|t| t.name() == "browser");

        let mut out = String::from(
            "## Periodic Monitoring & Scheduling\n\n\
             When the user requests periodic, recurring, or scheduled monitoring \
             (e.g. \"check this website every day\", \"notify me when X changes\", \
             \"monitor Y every hour\"), use the `cron_add` tool to create a scheduled agent job.\n\n\
             ### How to create a monitoring job\n\n\
             Use `cron_add` with `job_type: \"agent\"` and a descriptive `prompt` that tells \
             the scheduled agent exactly what to check and how to report results.\n\n\
             Required parameters:\n\
             - `schedule`: `{\"kind\":\"cron\",\"expr\":\"0 9 * * *\"}` (cron), \
             `{\"kind\":\"every\",\"every_ms\":3600000}` (interval), or \
             `{\"kind\":\"at\",\"at\":\"2025-01-01T09:00:00Z\"}` (one-shot)\n\
             - `job_type`: `\"agent\"`\n\
             - `prompt`: Clear instructions for the agent job\n\n\
             Optional parameters:\n\
             - `name`: Human-readable job name\n\
             - `delivery`: To send results to a channel, use:\n\
             ```json\n\
             {\"mode\":\"announce\",\"channel\":\"telegram\",\"to\":\"<chat_id>\"}\n\
             ```\n\
             Supported delivery channels: telegram, discord, slack, kakao, matrix, signal, \
             whatsapp, email, lark, dingtalk, qq, irc\n\
             - `session_target`: `\"isolated\"` (default, separate context) or `\"main\"`\n\
             - `model`: Override model for the job\n",
        );

        if has_browser {
            out.push_str(
                "\n### Browser access\n\n\
                 Scheduled agent jobs have full access to the `browser` tool for web scraping \
                 and monitoring. Include browser instructions in the job prompt when the task \
                 requires visiting websites.\n",
            );
        }

        out.push_str(
            "\n### Example\n\n\
             User: \"Check the Supreme Court website daily for new case rulings and notify me on Telegram.\"\n\n\
             Response: Use `cron_add` with:\n\
             ```json\n\
             {\n\
               \"name\": \"court_ruling_monitor\",\n\
               \"schedule\": {\"kind\": \"cron\", \"expr\": \"0 9 * * *\"},\n\
               \"job_type\": \"agent\",\n\
               \"prompt\": \"Use the browser tool to visit the Supreme Court website. Check for new case rulings published today. Summarize any new rulings found.\",\n\
               \"delivery\": {\"mode\": \"announce\", \"channel\": \"telegram\", \"to\": \"<chat_id>\"}\n\
             }\n\
             ```\n",
        );

        Ok(out)
    }
}

impl PromptSection for SkillsSection {
    fn name(&self) -> &str {
        "skills"
    }

    fn build(&self, ctx: &PromptContext<'_>) -> Result<String> {
        if ctx.skills.is_empty() {
            return Ok(String::new());
        }

        let mut prompt = String::from("## Available Skills\n\n<available_skills>\n");
        for skill in ctx.skills {
            let location = skill.location.clone().unwrap_or_else(|| {
                ctx.workspace_dir
                    .join("skills")
                    .join(&skill.name)
                    .join("SKILL.md")
            });
            let _ = writeln!(
                prompt,
                "  <skill>\n    <name>{}</name>\n    <description>{}</description>\n    <location>{}</location>\n  </skill>",
                skill.name,
                skill.description,
                location.display()
            );
        }
        prompt.push_str("</available_skills>");
        Ok(prompt)
    }
}

impl PromptSection for WorkspaceSection {
    fn name(&self) -> &str {
        "workspace"
    }

    fn build(&self, ctx: &PromptContext<'_>) -> Result<String> {
        Ok(format!(
            "## Workspace\n\nWorking directory: `{}`",
            ctx.workspace_dir.display()
        ))
    }
}

impl PromptSection for RuntimeSection {
    fn name(&self) -> &str {
        "runtime"
    }

    fn build(&self, ctx: &PromptContext<'_>) -> Result<String> {
        let host =
            hostname::get().map_or_else(|_| "unknown".into(), |h| h.to_string_lossy().to_string());
        Ok(format!(
            "## Runtime\n\nHost: {host} | OS: {} | Model: {}",
            std::env::consts::OS,
            ctx.model_name
        ))
    }
}

impl PromptSection for DateTimeSection {
    fn name(&self) -> &str {
        "datetime"
    }

    fn build(&self, _ctx: &PromptContext<'_>) -> Result<String> {
        let now = Local::now();
        Ok(format!(
            "## Current Date & Time\n\nTimezone: {}",
            now.format("%Z")
        ))
    }
}

fn inject_workspace_file(prompt: &mut String, workspace_dir: &Path, filename: &str) {
    let path = workspace_dir.join(filename);
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            let trimmed = content.trim();
            if trimmed.is_empty() {
                return;
            }
            let _ = writeln!(prompt, "### {filename}\n");
            let truncated = if trimmed.chars().count() > BOOTSTRAP_MAX_CHARS {
                trimmed
                    .char_indices()
                    .nth(BOOTSTRAP_MAX_CHARS)
                    .map(|(idx, _)| &trimmed[..idx])
                    .unwrap_or(trimmed)
            } else {
                trimmed
            };
            prompt.push_str(truncated);
            if truncated.len() < trimmed.len() {
                let _ = writeln!(
                    prompt,
                    "\n\n[... truncated at {BOOTSTRAP_MAX_CHARS} chars — use `read` for full file]\n"
                );
            } else {
                prompt.push_str("\n\n");
            }
        }
        Err(_) => {
            let _ = writeln!(prompt, "### {filename}\n\n[File not found: {filename}]\n");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::traits::Tool;
    use async_trait::async_trait;

    struct TestTool;

    #[async_trait]
    impl Tool for TestTool {
        fn name(&self) -> &str {
            "test_tool"
        }

        fn description(&self) -> &str {
            "tool desc"
        }

        fn parameters_schema(&self) -> serde_json::Value {
            serde_json::json!({"type": "object"})
        }

        async fn execute(
            &self,
            _args: serde_json::Value,
        ) -> anyhow::Result<crate::tools::ToolResult> {
            Ok(crate::tools::ToolResult {
                success: true,
                output: "ok".into(),
                error: None,
            })
        }
    }

    #[test]
    fn prompt_builder_assembles_sections() {
        let tools: Vec<Box<dyn Tool>> = vec![Box::new(TestTool)];
        let ctx = PromptContext {
            workspace_dir: Path::new("/tmp"),
            model_name: "test-model",
            tools: &tools,
            skills: &[],
            identity_config: None,
            dispatcher_instructions: "instr",
        };
        let prompt = SystemPromptBuilder::with_defaults().build(&ctx).unwrap();
        assert!(prompt.contains("## Tools"));
        assert!(prompt.contains("test_tool"));
        assert!(prompt.contains("instr"));
    }
}
