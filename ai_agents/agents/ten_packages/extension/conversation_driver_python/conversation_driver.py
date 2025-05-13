from dataclasses import dataclass
# from ten.async_ten_env import AsyncTenEnv
from ten_ai_base.config import BaseConfig
import datetime
import time

from ten import (
    AsyncTenEnv,
    Data,
)

DATA_OUT_TEXT_DATA_PROPERTY_TEXT = "text"

@dataclass
class ConversationDriverConfig(BaseConfig):
    greeting = "你好！我是您的生活秘书。请问有什么我可以帮助您的吗？"
    goodbye_keywords = ["bye", "再见", "goodbye"]
    goodbye = "再见期待下次再见"
    silence_timeout = 180    #AI说话之后，用户没有回复的静音超时
    scheduled_greetings = []


class ConversationDriver:
    def __init__(self, config: ConversationDriverConfig, ten_env: AsyncTenEnv) -> None:
        self.config = config
        self.ten_env = ten_env
        self.session = []
        self.waiting_for_reply = False
        self.last_response_time = time.time()
        self._last_triggered_minute = set()

    async def greet_user(self, data, **kwargs):
        # Send greeting message to the user
        self.ten_env.log_info(f"conversation_driver.greet_user() [{self.config.greeting}]")
        
        await self._send_text(self.config.greeting)
        self.session.append({"role": "assistant", "text": self.config.greeting})

        # After assistant greeting, wait for user reply
        self.waiting_for_reply = True
        self.last_response_time = time.time()

    async def on_user_input(self, msg, **kwargs):
        # Record user message
        self.session.append({"role": "user", "text": msg})
        self.last_response_time = time.time()
        # User has responded, so no longer waiting for user (waiting for LLM reply now)
        self.waiting_for_reply = False
        # Check if user said a goodbye keyword
        if any(keyword in msg.lower() for keyword in self.config.goodbye_keywords):
            self.config.goodbye = "Goodbye! Talk to you next time."
            await self._send_text(self.config.goodbye)
            self._reset_session()
            return

    def on_llm_reply(self, msg, **kwargs):
        # Record assistant (LLM) reply
        self.session.append({"role": "assistant", "text": msg})
        # After assistant reply, waiting for user response
        self.last_response_time = time.time()
        self.waiting_for_reply = True

    def on_user_left(self, data, **kwargs):
        # Reset the conversation session when the user leaves
        self._reset_session()

    # TODO 未完成
    async def tick(self):
        now = datetime.datetime.now()
        current_minute = now.strftime("%H:%M")
        # === 检查定时主动发言 ===
        for entry in self.config.scheduled_greetings:
            if entry.get("time") == current_minute and current_minute not in self._last_triggered_minute:
                self._log(f"Scheduled message triggered at {current_minute}")
                await self._send_text(entry.get("text", ""))
                self._last_triggered_minute.add(current_minute)
        # 清理已触发记录（仅保留当前小时内）
        self._last_triggered_minute = {
            t for t in self._last_triggered_minute if t >= now.strftime("%H:00")
        }

        # === 静音超时逻辑 ===
        if self.waiting_for_reply and time.time() - self.last_response_time > self.config.silence_timeout:
            await self._send_text("You've been quiet. I'm always here if you want to talk!")
            self._reset_session()

    async def _send_text(self, text):
        stable_data = Data.create("text_data")
        stable_data.set_property_string(DATA_OUT_TEXT_DATA_PROPERTY_TEXT, text)
        await self.ten_env.send_data(stable_data)

    def _reset_session(self):
        self.session.clear()
        self.waiting_for_reply = False

