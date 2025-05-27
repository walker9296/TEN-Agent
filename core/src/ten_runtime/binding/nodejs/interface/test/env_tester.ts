//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { Cmd } from "../msg/cmd.js";
import { CmdResult } from "../msg/cmd_result.js";
import { Data } from "../msg/data.js";
import { VideoFrame } from "../msg/video_frame.js";
import { AudioFrame } from "../msg/audio_frame.js";
import ten_addon from "../ten_addon.js";
import { LogLevel } from "../ten_env/log_level.js";

export class TenEnvTester {
  async sendCmd(cmd: Cmd): Promise<[CmdResult | null, Error | null]> {
    return new Promise<[CmdResult | null, Error | null]>((resolve) => {
      const err = ten_addon.ten_nodejs_ten_env_tester_send_cmd(
        this,
        cmd,
        async (cmdResult: CmdResult | null, error: Error | null) => {
          resolve([cmdResult, error]);
        },
      );

      if (err) {
        resolve([null, err]);
      }
    });
  }

  async sendData(data: Data): Promise<Error | null> {
    return new Promise<Error | null>((resolve) => {
      const err = ten_addon.ten_nodejs_ten_env_tester_send_data(
        this,
        data,
        async (error: Error | null) => {
          resolve(error);
        },
      );

      if (err) {
        resolve(err);
      }
    });
  }

  async sendVideoFrame(videoFrame: VideoFrame): Promise<Error | null> {
    return new Promise<Error | null>((resolve) => {
      const err = ten_addon.ten_nodejs_ten_env_tester_send_video_frame(
        this,
        videoFrame,
        async (error: Error | null) => {
          resolve(error);
        },
      );

      if (err) {
        resolve(err);
      }
    });
  }

  async sendAudioFrame(audioFrame: AudioFrame): Promise<Error | null> {
    return new Promise<Error | null>((resolve) => {
      const err = ten_addon.ten_nodejs_ten_env_tester_send_audio_frame(
        this,
        audioFrame,
        async (error: Error | null) => {
          resolve(error);
        },
      );

      if (err) {
        resolve(err);
      }
    });
  }

  async returnResult(cmdResult: CmdResult): Promise<Error | null> {
    return new Promise<Error | null>((resolve) => {
      const err = ten_addon.ten_nodejs_ten_env_tester_return_result(
        this,
        cmdResult,
        async (error: Error | null) => {
          resolve(error);
        },
      );

      if (err) {
        resolve(err);
      }
    });
  }

  stopTest(): Error | null {
    return ten_addon.ten_nodejs_ten_env_tester_stop_test(this);
  }

  logVerbose(message: string): Error | null {
    return this.log_internal(LogLevel.VERBOSE, message);
  }

  logDebug(message: string): Error | null {
    return this.log_internal(LogLevel.DEBUG, message);
  }

  logInfo(message: string): Error | null {
    return this.log_internal(LogLevel.INFO, message);
  }

  logWarn(message: string): Error | null {
    return this.log_internal(LogLevel.WARN, message);
  }

  logError(message: string): Error | null {
    return this.log_internal(LogLevel.ERROR, message);
  }

  logFatal(message: string): Error | null {
    return this.log_internal(LogLevel.FATAL, message);
  }

  private log_internal(level: number, message: string): Error | null {
    const _prepareStackTrace = Error.prepareStackTrace;
    Error.prepareStackTrace = (_, stack): NodeJS.CallSite[] => stack;
    const stack_ = new Error().stack as unknown as NodeJS.CallSite[];
    const stack = stack_.slice(1);
    Error.prepareStackTrace = _prepareStackTrace;

    const _callerFile = stack[1].getFileName();
    const _callerLine = stack[1].getLineNumber();
    const _callerFunction = stack[1].getFunctionName();

    const callerFile = _callerFile ? _callerFile : "unknown";
    const callerLine = _callerLine ? _callerLine : 0;
    const callerFunction = _callerFunction ? _callerFunction : "anonymous";

    return ten_addon.ten_nodejs_ten_env_tester_log_internal(
      this,
      level,
      callerFunction,
      callerFile,
      callerLine,
      message,
    );
  }
}

ten_addon.ten_nodejs_ten_env_tester_register_class(TenEnvTester);
