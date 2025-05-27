//
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0.
// See the LICENSE file for more information.
//
import {
  Addon,
  RegisterAddonAsExtension,
  Extension,
  TenEnv,
  Cmd,
  CmdResult,
  StatusCode,
  VideoFrame,
  Data,
  AudioFrame,
} from "ten-runtime-nodejs";

class DefaultExtension extends Extension {
  constructor(name: string) {
    super(name);
  }

  async onConfigure(_tenEnv: TenEnv): Promise<void> {
    console.log("DefaultExtension onConfigure");
  }

  async onInit(_tenEnv: TenEnv): Promise<void> {
    console.log("DefaultExtension onInit");
  }

  async onStart(tenEnv: TenEnv): Promise<void> {
    console.log("DefaultExtension onStart");

    const greetingCmd = Cmd.Create("greeting");

    const [greetingMsg] = await tenEnv.getPropertyString("greetingMsg");

    if (greetingMsg) {
      greetingCmd.setPropertyString("greetingMsg", greetingMsg);
    }

    tenEnv.sendCmd(greetingCmd);
  }

  async onCmd(tenEnv: TenEnv, cmd: Cmd): Promise<void> {
    const cmdName = cmd.getName();

    console.log("DefaultExtension onCmd" + cmdName);

    if (cmdName === "ping") {
      const cmdResult = CmdResult.Create(StatusCode.OK, cmd);
      tenEnv.returnResult(cmdResult);

      const pongCmd = Cmd.Create("pong");
      tenEnv.sendCmd(pongCmd);
    } else {
      const cmdResult = CmdResult.Create(StatusCode.ERROR, cmd);
      cmdResult.setPropertyString("detail", "unknown command");
      tenEnv.returnResult(cmdResult);
    }
  }

  async onData(tenEnv: TenEnv, data: Data): Promise<void> {
    console.log("DefaultExtension onData");

    const dataName = data.getName();
    if (dataName === "ping") {
      const pongData = Data.Create("pong");
      tenEnv.sendData(pongData);
    } else {
      tenEnv.logError("unknown data received: " + dataName);
    }
  }

  async onVideoFrame(tenEnv: TenEnv, videoFrame: VideoFrame): Promise<void> {
    console.log("DefaultExtension onVideoFrame");

    const videoFrameName = videoFrame.getName();
    if (videoFrameName === "ping") {
      const pongVideoFrame = VideoFrame.Create("pong");
      tenEnv.sendVideoFrame(pongVideoFrame);
    } else {
      tenEnv.logError("unknown video frame received: " + videoFrameName);
    }
  }

  async onAudioFrame(tenEnv: TenEnv, audioFrame: AudioFrame): Promise<void> {
    console.log("DefaultExtension onAudioFrame");

    const audioFrameName = audioFrame.getName();
    if (audioFrameName === "ping") {
      const pongAudioFrame = AudioFrame.Create("pong");
      tenEnv.sendAudioFrame(pongAudioFrame);
    } else {
      tenEnv.logError("unknown audio frame received: " + audioFrameName);
    }
  }

  async onStop(_tenEnv: TenEnv): Promise<void> {
    console.log("DefaultExtension onStop");
  }

  async onDeinit(_tenEnv: TenEnv): Promise<void> {
    console.log("DefaultExtension onDeinit");
  }
}

@RegisterAddonAsExtension("default_extension_nodejs")
class DefaultExtensionAddon extends Addon {
  async onCreateInstance(
    _tenEnv: TenEnv,
    instanceName: string,
  ): Promise<Extension> {
    return new DefaultExtension(instanceName);
  }
}
