pub mod roles;
pub mod bcip_roles;

pub use roles::{AgentRegistry, AgentRoleConfig, PatentAgentRole};
pub use bcip_roles::{patent_agent_role_configs, config_file_contents};
