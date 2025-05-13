from ten import (
    Addon,
    register_addon_as_extension,
    TenEnv,
)


@register_addon_as_extension("conversation_driver_python")
class ConversationDriverExtensionAddon(Addon):
    def on_create_instance(self, ten_env: TenEnv, name: str, context) -> None:
        from .extension import ConversationDriverExtension

        # ten_env.log_info("----------------------on_create_instance")
        ten_env.on_create_instance_done(ConversationDriverExtension(name), context)
