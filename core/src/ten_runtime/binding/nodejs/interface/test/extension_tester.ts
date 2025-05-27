//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { AudioFrame } from "../msg/audio_frame.js";
import { Data } from "../msg/data.js";
import { Cmd } from "../msg/cmd.js";
import { VideoFrame } from "../msg/video_frame.js";
import ten_addon from "../ten_addon.js";
import { TenEnvTester } from "./env_tester.js";

export class ExtensionTester {
  constructor() {
    ten_addon.ten_nodejs_extension_tester_create(this);
  }

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  async onStart(tenEnvTester: TenEnvTester): Promise<void> {
    // Stub for override.
  }

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  async onStop(tenEnvTester: TenEnvTester): Promise<void> {
    // Stub for override.
  }

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  async onDeinit(tenEnvTester: TenEnvTester): Promise<void> {
    // Stub for override.
  }

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  async onCmd(tenEnvTester: TenEnvTester, cmd: Cmd): Promise<void> {
    // Stub for override.
  }

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  async onData(tenEnvTester: TenEnvTester, data: Data): Promise<void> {
    // Stub for override.
  }

  async onAudioFrame(
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    tenEnvTester: TenEnvTester,
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    audioFrame: AudioFrame,
  ): Promise<void> {
    // Stub for override.
  }

  async onVideoFrame(
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    tenEnvTester: TenEnvTester,
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    videoFrame: VideoFrame,
  ): Promise<void> {
    // Stub for override.
  }

  async run(): Promise<void> {
    return ten_addon.ten_nodejs_extension_tester_run(this);
  }

  setTestModeSingle(addonName: string, propertyJsonStr: string): Error | null {
    return ten_addon.ten_nodejs_extension_tester_set_test_mode_single(
      this,
      addonName,
      propertyJsonStr,
    );
  }

  private async onInitProxy(tenEnvTester: TenEnvTester): Promise<void> {
    ten_addon.ten_nodejs_ten_env_tester_on_init_done(tenEnvTester);
  }

  private async onStartProxy(tenEnvTester: TenEnvTester): Promise<void> {
    await this.onStart(tenEnvTester);

    ten_addon.ten_nodejs_ten_env_tester_on_start_done(tenEnvTester);
  }

  private async onStopProxy(tenEnvTester: TenEnvTester): Promise<void> {
    await this.onStop(tenEnvTester);

    ten_addon.ten_nodejs_ten_env_tester_on_stop_done(tenEnvTester);
  }

  private async onDeinitProxy(tenEnvTester: TenEnvTester): Promise<void> {
    await this.onDeinit(tenEnvTester);

    ten_addon.ten_nodejs_ten_env_tester_on_deinit_done(tenEnvTester);

    // JS extension_tester prepare to be destroyed, so notify the underlying C
    // runtime this fact.
    ten_addon.ten_nodejs_extension_tester_on_end_of_life(this);

    (global as unknown as { gc: () => void }).gc();
  }

  private async onCmdProxy(
    tenEnvTester: TenEnvTester,
    cmd: Cmd,
  ): Promise<void> {
    await this.onCmd(tenEnvTester, cmd);
  }

  private async onDataProxy(
    tenEnvTester: TenEnvTester,
    data: Data,
  ): Promise<void> {
    await this.onData(tenEnvTester, data);
  }

  private async onAudioFrameProxy(
    tenEnvTester: TenEnvTester,
    audioFrame: AudioFrame,
  ): Promise<void> {
    await this.onAudioFrame(tenEnvTester, audioFrame);
  }

  private async onVideoFrameProxy(
    tenEnvTester: TenEnvTester,
    videoFrame: VideoFrame,
  ): Promise<void> {
    await this.onVideoFrame(tenEnvTester, videoFrame);
  }
}
