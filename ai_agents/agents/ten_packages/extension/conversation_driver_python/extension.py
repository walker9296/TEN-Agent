import asyncio
from ten import (
    AsyncExtension,
    AsyncTenEnv,
    Cmd,
    StatusCode,
    CmdResult,
    Data,
)
from .conversation_driver import ConversationDriver, ConversationDriverConfig

CMD_IN_FLUSH = "flush"
CMD_IN_ON_USER_JOINED = "on_user_joined"
CMD_IN_ON_USER_LEFT = "on_user_left"
CMD_OUT_FLUSH = "flush"

DATA_IN_TEXT_DATA_PROPERTY_IS_FINAL = "is_final"
DATA_IN_TEXT_DATA_PROPERTY_TEXT = "text"
DATA_IN_TEXT_DATA_PROPERTY_STREAM_ID = "stream_id"
DATA_IN_TEXT_DATA_PROPERTY_END_OF_SEGMENT = "end_of_segment"
DATA_OUT_TEXT_DATA_PROPERTY_TEXT = "text"

TICK_TIMEOUT = 60  # 秒，tick()定时器间隔

class ConversationDriverExtension(AsyncExtension):
    def __init__(self, name: str) -> None:
        super().__init__(name)
        self.client = None
        self.config = None

    async def on_init(self, ten_env: AsyncTenEnv) -> None:
        ten_env.log_info("ConversationDriverExtension on_init")

    async def on_start(self, ten_env: AsyncTenEnv) -> None:
        ten_env.log_info("ConversationDriverExtension on_start")
        self.config = await ConversationDriverConfig.create_async(ten_env=ten_env)
        self.client = ConversationDriver(self.config, ten_env=ten_env)

        #TODO
        # 主动发话的定时器
        # asyncio.create_task(self._tick_loop())

        
    async def on_stop(self, ten_env: AsyncTenEnv) -> None:
        ten_env.log_info("ConversationDriverExtension on_stop")
        # if self.client:
        #     await self.client.finish()

    async def on_deinit(self, ten_env: AsyncTenEnv) -> None:
        ten_env.log_info("ConversationDriverExtension on_deinit")

    async def on_cmd(self, ten_env: AsyncTenEnv, cmd: Cmd) -> None:
        cmd_name = cmd.get_name()
        ten_env.log_info("ConversationDriverExtension on_cmd() name {}".format(cmd_name))

        if cmd_name == CMD_IN_FLUSH:
            pass
        elif cmd_name == CMD_IN_ON_USER_JOINED:
            await self.client.greet_user(Cmd)
        else:
            self.client.on_user_left(Cmd)

        # Create a CmdResult object to return
        cmd_result = CmdResult.create(StatusCode.OK)
        await ten_env.return_result(cmd_result, cmd)

    async def on_data(self, ten_env: AsyncTenEnv, data: Data) -> None:
        data_name = data.get_name()
        ten_env.log_info("ConversationDriverExtension on_data() name {}".format(data_name))

        try:
            input_text = data.get_property_string(DATA_IN_TEXT_DATA_PROPERTY_TEXT)
        except Exception as err:
            ten_env.log_info(f"GetProperty optional {DATA_IN_TEXT_DATA_PROPERTY_TEXT} failed, err: {err}")

        try:
            # 从sst中能够获取stream_id属性，而从LLM中获取不到
            stream_id = data.get_property_int(DATA_IN_TEXT_DATA_PROPERTY_STREAM_ID)
            is_final = data.get_property_bool(DATA_IN_TEXT_DATA_PROPERTY_IS_FINAL)
            if is_final == True:
                await self.client.on_user_input(input_text)
                ten_env.log_info(f"ConversationDriverExtension on_user_input: [{input_text}]")
        except Exception as err:
            self.client.on_llm_reply(input_text)
            # ten_env.log_info(f"ConversationDriverExtension on_data: [{err}]")
            ten_env.log_info(f"ConversationDriverExtension on_llm_reply: [{input_text}]")
        
    async def _tick_loop(self):
        while True:
            try:
                await self.client.tick()
            except Exception as e:
                self.ten_env.log_info(f"ConversationDriverExtension _tick_loop: [{e}]")
            await asyncio.sleep(TICK_TIMEOUT)  # 每TICK_TIMEOUT秒调用一次