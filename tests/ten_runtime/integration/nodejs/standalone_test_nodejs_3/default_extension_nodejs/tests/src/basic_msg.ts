//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import {
  AudioFrame,
  Cmd,
  CmdResult,
  Data,
  ExtensionTester,
  StatusCode,
  TenEnvTester,
  VideoFrame,
} from "ten-runtime-nodejs";

export class CmdTester extends ExtensionTester {
  async onStart(tenEnvTester: TenEnvTester) {
    tenEnvTester.logInfo("CmdTester onStart");

    const pingCmd = Cmd.Create("ping");
    tenEnvTester.sendCmd(pingCmd);
  }

  async onCmd(tenEnvTester: TenEnvTester, cmd: Cmd) {
    const cmdName = cmd.getName();
    console.log("CmdTester onCmd: " + cmdName);

    if (cmdName === "pong") {
      tenEnvTester.logInfo("pong cmd received");

      const cmdResult = CmdResult.Create(StatusCode.OK, cmd);
      await tenEnvTester.returnResult(cmdResult);

      tenEnvTester.stopTest();
    }
  }
}

export class DataTester extends ExtensionTester {
  async onStart(tenEnvTester: TenEnvTester) {
    tenEnvTester.logInfo("DataTester onStart");

    const pingData = Data.Create("ping");
    tenEnvTester.sendData(pingData);
  }

  async onData(tenEnvTester: TenEnvTester, data: Data) {
    const dataName = data.getName();
    console.log("DataTester onData: " + dataName);

    if (dataName === "pong") {
      tenEnvTester.logInfo("pong data received");

      tenEnvTester.stopTest();
    }
  }
}

export class VideoFrameTester extends ExtensionTester {
  async onStart(tenEnvTester: TenEnvTester) {
    tenEnvTester.logInfo("VideoFrameTester onStart");

    const pingVideoFrame = VideoFrame.Create("ping");
    tenEnvTester.sendVideoFrame(pingVideoFrame);
  }

  async onVideoFrame(tenEnvTester: TenEnvTester, videoFrame: VideoFrame) {
    const videoFrameName = videoFrame.getName();
    console.log("VideoFrameTester onVideoFrame: " + videoFrameName);

    if (videoFrameName === "pong") {
      tenEnvTester.logInfo("pong video frame received");

      tenEnvTester.stopTest();
    }
  }
}

export class AudioFrameTester extends ExtensionTester {
  async onStart(tenEnvTester: TenEnvTester) {
    tenEnvTester.logInfo("AudioFrameTester onStart");

    const pingAudioFrame = AudioFrame.Create("ping");
    tenEnvTester.sendAudioFrame(pingAudioFrame);
  }

  async onAudioFrame(tenEnvTester: TenEnvTester, audioFrame: AudioFrame) {
    const audioFrameName = audioFrame.getName();
    console.log("AudioFrameTester onAudioFrame: " + audioFrameName);

    if (audioFrameName === "pong") {
      tenEnvTester.logInfo("pong audio frame received");

      tenEnvTester.stopTest();
    }
  }
}
