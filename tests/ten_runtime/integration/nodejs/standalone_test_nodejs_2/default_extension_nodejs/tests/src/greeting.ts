//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import {
  Cmd,
  CmdResult,
  ExtensionTester,
  StatusCode,
  TenEnvTester,
} from "ten-runtime-nodejs";

export class GreetingTester extends ExtensionTester {
  private expectedGreetingMsg: string;

  constructor(expectedGreetingMsg: string) {
    super();
    this.expectedGreetingMsg = expectedGreetingMsg;
  }

  async onStart(tenEnvTester: TenEnvTester) {
    console.log("GreetingTester onStart");
  }

  async onStop(tenEnvTester: TenEnvTester) {
    console.log("GreetingTester onStop");
  }

  async onCmd(tenEnvTester: TenEnvTester, cmd: Cmd) {
    const cmdName = cmd.getName();
    console.log("GreetingTester onCmd: " + cmdName);

    if (cmdName === "greeting") {
      const [actualGreetingMsg, err] = cmd.getPropertyString("greetingMsg");
      if (err) {
        throw new Error(`Failed to get greeting message: ${err.message}`);
      }

      if (actualGreetingMsg !== this.expectedGreetingMsg) {
        throw new Error(
          `Expected greeting message: ${this.expectedGreetingMsg}, but got: ${actualGreetingMsg}`,
        );
      }

      const cmdResult = CmdResult.Create(StatusCode.OK, cmd);
      await tenEnvTester.returnResult(cmdResult);

      tenEnvTester.stopTest();
    }
  }

  async onDeinit(tenEnvTester: TenEnvTester) {
    console.log("GreetingTester onDeinit");
  }
}
