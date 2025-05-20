//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//

package ten

// #include "ten_env_tester.h"
import "C"
import (
	"runtime"
	"strings"
	"unsafe"
)

type (
	// TesterResultHandler is the handler for the result of the command.
	TesterResultHandler func(TenEnvTester, CmdResult, error)

	// TesterErrorHandler is the handler for the error of the command.
	TesterErrorHandler func(TenEnvTester, error)
)

// TenEnvTester is the interface for the ten env tester.
type TenEnvTester interface {
	OnStartDone() error
	OnStopDone() error
	OnDeinitDone() error

	SendCmd(cmd Cmd, handler TesterResultHandler) error
	SendCmdEx(cmd Cmd, handler TesterResultHandler) error
	SendData(data Data, handler TesterErrorHandler) error
	SendAudioFrame(audioFrame AudioFrame, handler TesterErrorHandler) error
	SendVideoFrame(videoFrame VideoFrame, handler TesterErrorHandler) error

	ReturnResult(result CmdResult, handler TesterErrorHandler) error

	StopTest() error

	LogVerbose(msg string) error
	LogDebug(msg string) error
	LogInfo(msg string) error
	LogWarn(msg string) error
	LogError(msg string) error
	LogFatal(msg string) error
}

var (
	_ TenEnvTester = new(tenEnvTester)
)

type tenEnvTester struct {
	baseTenObject[C.uintptr_t]
}

func (p *tenEnvTester) OnStartDone() error {
	return withCGOLimiter(func() error {
		cStatus := C.ten_go_ten_env_tester_on_start_done(p.cPtr)
		return withCGoError(&cStatus)
	})
}

func (p *tenEnvTester) OnStopDone() error {
	return withCGOLimiter(func() error {
		cStatus := C.ten_go_ten_env_tester_on_stop_done(p.cPtr)
		return withCGoError(&cStatus)
	})
}

func (p *tenEnvTester) OnDeinitDone() error {
	return withCGOLimiter(func() error {
		cStatus := C.ten_go_ten_env_tester_on_deinit_done(p.cPtr)
		return withCGoError(&cStatus)
	})
}

func (p *tenEnvTester) SendCmd(cmd Cmd, handler TesterResultHandler) error {
	if cmd == nil {
		return newTenError(
			ErrorCodeInvalidArgument,
			"cmd is required.",
		)
	}

	return withCGOLimiter(func() error {
		return p.sendCmd(cmd, handler)
	})
}

func (p *tenEnvTester) SendCmdEx(cmd Cmd, handler TesterResultHandler) error {
	if cmd == nil {
		return newTenError(
			ErrorCodeInvalidArgument,
			"cmd is required.",
		)
	}

	return withCGOLimiter(func() error {
		return p.sendCmdEx(cmd, handler)
	})
}

func (p *tenEnvTester) SendData(data Data, handler TesterErrorHandler) error {
	if data == nil {
		return newTenError(
			ErrorCodeInvalidArgument,
			"data is required.",
		)
	}

	return withCGOLimiter(func() error {
		return p.sendData(data, handler)
	})
}

func (p *tenEnvTester) SendAudioFrame(
	audioFrame AudioFrame,
	handler TesterErrorHandler,
) error {
	if audioFrame == nil {
		return newTenError(
			ErrorCodeInvalidArgument,
			"audioFrame is required.",
		)
	}

	return withCGOLimiter(func() error {
		return p.sendAudioFrame(audioFrame, handler)
	})
}

func (p *tenEnvTester) SendVideoFrame(
	videoFrame VideoFrame,
	handler TesterErrorHandler,
) error {
	if videoFrame == nil {
		return newTenError(
			ErrorCodeInvalidArgument,
			"videoFrame is required.",
		)
	}

	return withCGOLimiter(func() error {
		return p.sendVideoFrame(videoFrame, handler)
	})
}

func (p *tenEnvTester) ReturnResult(
	result CmdResult,
	handler TesterErrorHandler,
) error {
	if result == nil {
		return newTenError(
			ErrorCodeInvalidArgument,
			"result is required.",
		)
	}

	return withCGOLimiter(func() error {
		return p.returnResult(result, handler)
	})
}

func (p *tenEnvTester) StopTest() error {
	return withCGOLimiter(func() error {
		return p.stopTest()
	})
}

func (p *tenEnvTester) sendCmd(cmd Cmd, handler TesterResultHandler) error {
	defer cmd.keepAlive()

	cb := goHandleNil
	if handler != nil {
		cb = newGoHandle(handler)
	}

	cStatus := C.ten_go_ten_env_tester_send_cmd(
		p.cPtr,
		cmd.getCPtr(),
		cHandle(cb),
		C.bool(false),
	)

	return withCGoError(&cStatus)
}

func (p *tenEnvTester) sendCmdEx(cmd Cmd, handler TesterResultHandler) error {
	defer cmd.keepAlive()

	cb := goHandleNil
	if handler != nil {
		cb = newGoHandle(handler)
	}

	cStatus := C.ten_go_ten_env_tester_send_cmd(
		p.cPtr,
		cmd.getCPtr(),
		cHandle(cb),
		C.bool(true),
	)

	return withCGoError(&cStatus)
}

func (p *tenEnvTester) sendData(data Data, handler TesterErrorHandler) error {
	defer data.keepAlive()

	cb := goHandleNil
	if handler != nil {
		cb = newGoHandle(handler)
	}

	cStatus := C.ten_go_ten_env_tester_send_data(
		p.cPtr,
		data.getCPtr(),
		cHandle(cb),
	)

	return withCGoError(&cStatus)
}

func (p *tenEnvTester) sendAudioFrame(
	audioFrame AudioFrame,
	handler TesterErrorHandler,
) error {
	defer audioFrame.keepAlive()

	cb := goHandleNil
	if handler != nil {
		cb = newGoHandle(handler)
	}

	cStatus := C.ten_go_ten_env_tester_send_audio_frame(
		p.cPtr,
		audioFrame.getCPtr(),
		cHandle(cb),
	)

	return withCGoError(&cStatus)
}

func (p *tenEnvTester) sendVideoFrame(
	videoFrame VideoFrame,
	handler TesterErrorHandler,
) error {
	defer videoFrame.keepAlive()

	cb := goHandleNil
	if handler != nil {
		cb = newGoHandle(handler)
	}

	cStatus := C.ten_go_ten_env_tester_send_video_frame(
		p.cPtr,
		videoFrame.getCPtr(),
		cHandle(cb),
	)

	return withCGoError(&cStatus)
}

func (p *tenEnvTester) returnResult(
	result CmdResult,
	handler TesterErrorHandler,
) error {
	if result == nil {
		return newTenError(
			ErrorCodeInvalidArgument,
			"result is required.",
		)
	}

	cb := goHandleNil
	if handler != nil {
		cb = newGoHandle(handler)
	}

	cStatus := C.ten_go_ten_env_tester_return_result(
		p.cPtr,
		result.getCPtr(),
		cHandle(cb),
	)

	return withCGoError(&cStatus)
}

func (p *tenEnvTester) stopTest() error {
	cStatus := C.ten_go_ten_env_tester_stop_test(p.cPtr)

	return withCGoError(&cStatus)
}

func (p *tenEnvTester) LogVerbose(msg string) error {
	return p.logInternal(LogLevelVerbose, msg, 2)
}

func (p *tenEnvTester) LogDebug(msg string) error {
	return p.logInternal(LogLevelDebug, msg, 2)
}

func (p *tenEnvTester) LogInfo(msg string) error {
	return p.logInternal(LogLevelInfo, msg, 2)
}

func (p *tenEnvTester) LogWarn(msg string) error {
	return p.logInternal(LogLevelWarn, msg, 2)
}

func (p *tenEnvTester) LogError(msg string) error {
	return p.logInternal(LogLevelError, msg, 2)
}

func (p *tenEnvTester) LogFatal(msg string) error {
	return p.logInternal(LogLevelFatal, msg, 2)
}

func (p *tenEnvTester) logInternal(level LogLevel, msg string, skip int) error {
	// Get caller info.
	pc, fileName, lineNo, ok := runtime.Caller(skip)
	funcName := "unknown"
	if ok {
		fn := runtime.FuncForPC(pc)
		if fn != nil {
			funcName = fn.Name()

			parts := strings.Split(funcName, ".")
			if len(parts) > 0 {
				// The last part is the method name.
				funcName = parts[len(parts)-1]
			}
		}
	} else {
		fileName = "unknown"
		lineNo = 0
	}

	cStatus := C.ten_go_ten_env_tester_log(
		p.cPtr,
		C.int(level),
		unsafe.Pointer(unsafe.StringData(funcName)),
		C.int(len(funcName)),
		unsafe.Pointer(unsafe.StringData(fileName)),
		C.int(len(fileName)),
		C.int(lineNo),
		unsafe.Pointer(unsafe.StringData(msg)),
		C.int(len(msg)),
	)

	return withCGoError(&cStatus)
}

//export tenGoCreateTenEnvTester
func tenGoCreateTenEnvTester(cInstance C.uintptr_t) C.uintptr_t {
	tenEnvTesterInstance := &tenEnvTester{}
	tenEnvTesterInstance.cPtr = cInstance
	tenEnvTesterInstance.pool = newJobPool(5)
	runtime.SetFinalizer(tenEnvTesterInstance, func(p *tenEnvTester) {
		C.ten_go_ten_env_tester_finalize(p.cPtr)
	})

	id := newhandle(tenEnvTesterInstance)
	tenEnvTesterInstance.goObjID = id

	return C.uintptr_t(id)
}
