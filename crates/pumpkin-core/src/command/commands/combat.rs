use pumpkin_config::CombatMode;
use pumpkin_util::{
    PermissionLvl,
    permission::{Permission, PermissionDefault, PermissionRegistry},
    text::TextComponent,
};

use crate::command::{
    argument_builder::{ArgumentBuilder, command, literal},
    context::command_context::CommandContext,
    node::{CommandExecutor, CommandExecutorResult, dispatcher::CommandDispatcher},
};

const DESCRIPTION: &str = "Shows or switches the combat mode (modern or legacy_1_8), live.";
const PERMISSION: &str = "pumpkin:command.combat";

const fn name(mode: CombatMode) -> &'static str {
    match mode {
        CombatMode::Modern => "modern",
        CombatMode::Legacy1_8 => "legacy_1_8",
    }
}

struct CombatQueryExecutor;

impl CommandExecutor for CombatQueryExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let mode = context.server().combat_mode();
        context.source.send_feedback(
            TextComponent::text(format!("Combat mode: {}", name(mode))),
            false,
        );
        Ok(i32::from(mode.is_legacy()))
    }
}

struct CombatSetExecutor(CombatMode);

impl CommandExecutor for CombatSetExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let server = context.server();
        server.set_combat_mode(self.0);
        let announce = TextComponent::text(format!(
            "Combat mode switched to {} by {}.",
            name(self.0),
            context.source.name
        ));
        for world in server.worlds.load().iter() {
            world.broadcast_system_message(&announce, false);
        }
        Ok(i32::from(self.0.is_legacy()))
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Three),
    ));

    dispatcher.register(
        command("combat", DESCRIPTION)
            .requires(PERMISSION)
            .then(literal("modern").executes(CombatSetExecutor(CombatMode::Modern)))
            .then(literal("legacy_1_8").executes(CombatSetExecutor(CombatMode::Legacy1_8)))
            .executes(CombatQueryExecutor),
    );
}
