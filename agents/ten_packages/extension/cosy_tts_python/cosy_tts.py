import asyncio
from dataclasses import dataclass

from websocket import WebSocketConnectionClosedException

from ten.async_ten_env import AsyncTenEnv
from ten_ai_base.config import BaseConfig
import dashscope
from dashscope.audio.tts_v2 import SpeechSynthesizer, AudioFormat, ResultCallback

import aiohttp

@dataclass
class CosyTTSConfig(BaseConfig):
    api_key: str = ""
#    ws_url: str = "ws://localhost:50001"
    base_url: str = ""
    voice: str = "longxiaochun"
    model: str = "cosyvoice-v1"
    sample_rate: int = 16000


class AsyncIteratorCallback(ResultCallback):
    def __init__(self, ten_env: AsyncTenEnv, queue: asyncio.Queue) -> None:
        self.closed = False
        self.ten_env = ten_env
        self.loop = asyncio.get_event_loop()
        self.queue = queue

    def close(self):
        self.closed = True

    def on_open(self):
        self.ten_env.log_info("websocket is open.")

    def on_complete(self):
        self.ten_env.log_info("speech synthesis task complete successfully.")

    def on_error(self, message: str):
        self.ten_env.log_error(f"speech synthesis task failed, {message}")

    def on_close(self):
        self.ten_env.log_info("websocket is closed.")
        self.close()

    def on_event(self, message: str) -> None:
        self.ten_env.log_debug(f"received event: {message}")

    def on_data(self, data: bytes) -> None:
        if self.closed:
            self.ten_env.log_warn(
                f"received data: {len(data)} bytes but connection was closed"
            )
            return
        self.ten_env.log_debug(f"received data: {len(data)} bytes")
        asyncio.run_coroutine_threadsafe(self.queue.put(data), self.loop)


class CosyTTS:
    def __init__(self, config: CosyTTSConfig) -> None:
        self.config = config
        self.synthesizer = None  # Initially no synthesizer
        self.queue = asyncio.Queue()
        if not config.base_url:
            dashscope.api_key = config.api_key

    def _create_synthesizer(
        self, ten_env: AsyncTenEnv, callback: AsyncIteratorCallback
    ):
        if self.synthesizer:
            self.synthesizer = None

        ten_env.log_info("Creating new synthesizer")
        self.synthesizer = SpeechSynthesizer(
            model=self.config.model,
            voice=self.config.voice,
            format=AudioFormat.PCM_16000HZ_MONO_16BIT,
            callback=callback,
        )

    async def get_audio_bytes(self) -> bytes:
        return await self.queue.get()

    def text_to_speech_stream(
        self, ten_env: AsyncTenEnv, text: str, end_of_segment: bool
    ) -> None:
        if self.config.base_url:
            asyncio.create_task(self._text_to_speech_rest(ten_env, text, end_of_segment))
        else:
            self._text_to_speech_dashscope(ten_env, text, end_of_segment)

    def _text_to_speech_dashscope(self, ten_env: AsyncTenEnv, text: str, end_of_segment: bool) -> None:
        try:
            callback = AsyncIteratorCallback(ten_env, self.queue)

            if not self.synthesizer or end_of_segment:
                self._create_synthesizer(ten_env, callback)

            self.synthesizer.streaming_call(text)

            if end_of_segment:
                ten_env.log_info("Streaming complete")
                self.synthesizer.streaming_complete()
                self.synthesizer = None
        except WebSocketConnectionClosedException as e:
            ten_env.log_error(f"WebSocket connection closed, {e}")
            self.synthesizer = None
        except Exception as e:
            ten_env.log_error(f"Error streaming text, {e}")
            self.synthesizer = None

    async def _text_to_speech_rest(self, ten_env: AsyncTenEnv, text: str, end_of_segment: bool):
        try:
            url = self.config.base_url.rstrip("/")
            payload = {"tts_text": text, "spk_id": self.config.voice}
            #ten_env.log_info(f"Calling REST TTS: {url} [{json.dumps(payload)}]")

            async with aiohttp.ClientSession() as session:
                async with session.post(url, data=payload) as resp:
                    if resp.status != 200:
                        detail = await resp.text()
                        ten_env.log_error(f"REST TTS failed, status: {resp.status}, detail: {detail}")
                        return

                    async for chunk in resp.content.iter_chunked(2048):
                        if chunk:
                            await self.queue.put(chunk)
        except Exception as e:
            ten_env.log_error(f"REST TTS error: {e}")


    def cancel(self, ten_env: AsyncTenEnv) -> None:
        if self.synthesizer:
            try:
                self.synthesizer.streaming_cancel()
            except WebSocketConnectionClosedException as e:
                ten_env.log_error(f"WebSocket connection closed, {e}")
            except Exception as e:
                ten_env.log_error(f"Error cancelling streaming, {e}")
            self.synthesizer = None
