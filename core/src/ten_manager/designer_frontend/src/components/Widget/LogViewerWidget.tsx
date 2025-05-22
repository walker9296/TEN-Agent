//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";
import { z } from "zod";
import { useTranslation } from "react-i18next";
import { VariableSizeList as VirtualList } from "react-window";
import AutoSizer from "react-virtualized-auto-sizer";

import { Input } from "@/components/ui/Input";
import { Button } from "@/components/ui/Button";
import { Combobox } from "@/components/ui/Combobox";
import { cn } from "@/lib/utils";
import { useAppStore, useFlowStore, useWidgetStore } from "@/store";
import { appendLogsById } from "@/store/widget";
import { ILogViewerWidget, ILogViewerWidgetOptions } from "@/types/widgets";
import {
  EWSMessageType,
  LogSchema,
  LegacyLogSchema,
  LogLineInfoSchema,
} from "@/types/apps";

export function LogViewerBackstageWidget(props: ILogViewerWidget) {
  const {
    widget_id: id,
    metadata: { wsUrl, scriptType, script, postActions } = {},
  } = props;

  const wsRef = React.useRef<WebSocket | null>(null);
  const hasConnectedRef = React.useRef(false);

  React.useEffect(() => {
    if (!wsUrl || !scriptType || !script) {
      return;
    }

    if (hasConnectedRef.current) {
      return;
    }

    wsRef.current = new WebSocket(wsUrl);
    hasConnectedRef.current = true;

    wsRef.current.onopen = () => {
      console.log("[LogViewerWidget] WebSocket connected!");
      wsRef.current?.send(JSON.stringify(script));
    };

    wsRef.current.onmessage = (event) => {
      try {
        const msg: z.infer<typeof LogSchema> | z.infer<typeof LegacyLogSchema> =
          JSON.parse(event.data);
        const isLegacy = typeof msg.data === "string";

        if (
          msg.type === EWSMessageType.STANDARD_OUTPUT_LOG ||
          msg.type === EWSMessageType.STANDARD_ERROR_LOG ||
          msg.type === EWSMessageType.NORMAL_LINE
        ) {
          if (isLegacy) {
            const line = (msg as z.infer<typeof LegacyLogSchema>).data;
            appendLogsById(id, [{ line }]);
          } else {
            const data = (msg as z.infer<typeof LogSchema>).data;
            appendLogsById(id, [data]);
          }
        } else if (msg.type === EWSMessageType.EXIT) {
          const code = msg.code;
          const errMsg = msg?.error_message;
          const lines = [];
          if (errMsg) {
            lines.push(errMsg);
          }
          lines.push(`Process exited with code ${code}. Closing...`);
          appendLogsById(
            id,
            lines.map((line) => ({ line }))
          );

          wsRef.current?.close();
        } else if (msg.status === "fail") {
          appendLogsById(id, [
            { line: `Error: ${msg.message || "Unknown error"}` },
          ]);
        } else {
          appendLogsById(id, [
            { line: `Unknown message: ${JSON.stringify(msg)}` },
          ]);
        }

        // eslint-disable-next-line @typescript-eslint/no-unused-vars
      } catch (err) {
        // If it's not JSON, output it directly as text.
        appendLogsById(id, [{ line: event.data }]);
      }
    };

    wsRef.current.onerror = (err) => {
      console.error("[LogViewerWidget] WebSocket error:", err);
    };

    wsRef.current.onclose = () => {
      console.log("[LogViewerWidget] WebSocket closed!");
      postActions?.();
    };

    return () => {
      // Close the connection when the component is unmounted.
      hasConnectedRef.current = false;
      wsRef.current?.close();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [id, wsUrl, scriptType, script]);

  return <></>;
}

export function LogViewerFrontStageWidget(props: {
  id: string;
  options?: ILogViewerWidgetOptions;
}) {
  const { id, options } = props;

  const [searchInput, setSearchInput] = React.useState("");
  const defferedSearchInput = React.useDeferredValue(searchInput);
  const [addonInput, setAddonInput] = React.useState(
    options?.filters?.extensions?.[0] || ""
  );

  const { logViewerHistory, widgets } = useWidgetStore();
  const { nodes } = useFlowStore();
  const { currentWorkspace } = useAppStore();

  const { t } = useTranslation();

  const logsMemo = React.useMemo(() => {
    if (options?.filters?.extensions?.length) {
      // Filter logs by selected extension
      const allLogs = options.filters.extensions.reduce(
        (acc, ext) => {
          // Get all logs from logViewerHistory
          const allLogs = Object.values(logViewerHistory).flatMap(
            (viewer) => viewer.history || []
          );
          return [
            ...acc,
            ...allLogs.filter((log) => log.metadata?.extension === ext),
          ];
        },
        [] as (typeof logViewerHistory)[typeof id]["history"]
      );
      return allLogs.filter(
        (log) =>
          log.metadata?.extension === addonInput &&
          currentWorkspace.graph?.name === log.metadata?.graph_name
      );
    }
    const allLogs = logViewerHistory[id]?.history || [];
    if (!addonInput) return allLogs;
    return (
      allLogs.filter((log) => log.metadata?.extension === addonInput) || []
    );
  }, [
    logViewerHistory,
    id,
    addonInput,
    currentWorkspace.graph?.name,
    options?.filters?.extensions,
  ]);

  const currentWidget = React.useMemo(() => {
    return widgets.find((w) => w.widget_id === id);
  }, [widgets, id]);

  return (
    <div className="flex h-full w-full flex-col" id={id}>
      {!options?.disableSearch && (
        <div className="h-12 w-full flex items-center space-x-2 px-2">
          <Input
            placeholder="Search"
            className="w-full"
            value={searchInput}
            onChange={(e) => setSearchInput(e.target.value)}
          />
          <Button variant="outline" className="hidden">
            {t("action.search")}
          </Button>
          {!options?.filters?.extensions && (
            <Combobox
              className="w-1/3"
              options={nodes.map((node) => ({
                label: node.data.name,
                value: node.data.name,
              }))}
              placeholder={t("popup.logViewer.filteredByAddon")}
              selected={addonInput}
              onChange={(i) => {
                if (i.value === addonInput) {
                  setAddonInput("");
                  return;
                }
                console.log("onChange", i.value);
                setAddonInput(i.value);
              }}
              commandLabels={{
                placeholder: t("popup.logViewer.filteredByAddon"),
                noItems: t("popup.logViewer.noAddons"),
                noMatchedItems: t("popup.logViewer.noMatchedAddons"),
              }}
            />
          )}
        </div>
      )}
      <div className="h-full w-full p-2">
        <LogViewerLogItemList
          logs={logsMemo}
          search={defferedSearchInput}
          prefix={`${id}-${currentWidget?.display_type}`}
        />
      </div>
    </div>
  );
}

const string2uuid = (str: string) => {
  // Create a deterministic hash from the string
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i);
    hash = (hash << 5) - hash + char;
    hash = hash & hash; // Convert to 32-bit integer
  }

  // Use the hash to create a deterministic UUID-like string
  const hashStr = Math.abs(hash).toString(16).padStart(8, "0");
  return (
    `${hashStr}-${hashStr.slice(0, 4)}` +
    `-4${hashStr.slice(4, 7)}-${hashStr.slice(7, 11)}-${hashStr.slice(0, 12)}`
  );
};

export interface ILogViewerLogItemProps {
  id: string;
  extension?: string;
  file?: string;
  line?: number;
  host?: string;
  message: string;
}

const string2LogItem = (str?: string): ILogViewerLogItemProps => {
  if (!str) {
    return {
      id: string2uuid(new Date().getTime().toString()),
      message: "",
    };
  }
  const regex = /^(\w+)@([^:]+):(\d+)\s+\[([^\]]+)\]\s+(.+)$/;
  const match = str.match(regex);
  const randomId = string2uuid(str + new Date().getTime());
  if (!match) {
    return {
      id: randomId,
      message: str,
    };
  }
  const [, extension, file, line, host, message] = match;
  return {
    id: randomId,
    extension,
    file,
    line: parseInt(line, 10),
    host,
    message,
  };
};

const LogViewerLogItem = React.forwardRef<
  HTMLDivElement,
  ILogViewerLogItemProps & { search?: string; className?: string }
>((props, ref) => {
  const { id, extension, file, line, host, message, search, className } = props;

  return (
    <div
      ref={ref}
      className={cn(
        "font-mono text-xs py-0.5",
        "hover:bg-gray-100 dark:hover:bg-gray-800",
        className
      )}
      id={id}
    >
      {extension && (
        <>
          <span className="text-blue-500 dark:text-blue-400">{extension}</span>
          <span className="text-gray-500 dark:text-gray-400">@</span>
        </>
      )}
      {file && (
        <>
          <span className="text-emerald-600 dark:text-emerald-400">{file}</span>
          <span className="text-gray-500 dark:text-gray-400">:</span>
        </>
      )}
      {line && (
        <span className="text-amber-600 dark:text-amber-400">{line}</span>
      )}
      {host && (
        <>
          <span className="text-gray-500 dark:text-gray-400"> [</span>
          <span className="text-purple-600 dark:text-purple-400">{host}</span>
          <span className="text-gray-500 dark:text-gray-400">] </span>
        </>
      )}
      {search ? (
        <span className="whitespace-pre-wrap">
          {message.split(search).map((part, i, arr) => (
            <React.Fragment key={i}>
              {part}
              {i < arr.length - 1 && (
                <span className="bg-yellow-200 dark:bg-yellow-800">
                  {search}
                </span>
              )}
            </React.Fragment>
          ))}
        </span>
      ) : (
        <span className="whitespace-pre-wrap">{message}</span>
      )}
    </div>
  );
});
LogViewerLogItem.displayName = "LogViewerLogItem";

const VirtualListItem = (props: {
  data: ILogViewerLogItemProps[];
  index: number;
  setSize: (index: number, size: number) => void;
  windowWidth: number;
  search?: string;
}) => {
  const { data, index, setSize, windowWidth, search } = props;

  const rowRef = React.useRef<HTMLDivElement>(null);

  React.useEffect(() => {
    if (rowRef.current) {
      setSize(index, rowRef.current.getBoundingClientRect().height);
    }
  }, [setSize, index, windowWidth]);

  return (
    <>
      {/* <div
        ref={rowRef}
        style={{
          whiteSpace: "pre-wrap",
          wordBreak: "break-word",
        }}
      >
        {(data[index] as ILogViewerLogItemProps).message}
      </div> */}
      <LogViewerLogItem
        ref={rowRef}
        {...(data[index] as ILogViewerLogItemProps)}
        search={search}
        className="w-full whitespace-break-spaces break-words break-all"
      />
    </>
  );
};

function LogViewerLogItemList(props: {
  logs: z.infer<typeof LogLineInfoSchema>[];
  search?: string;
  prefix?: string;
}) {
  const { logs: rawLogs, search, prefix } = props;

  const { t } = useTranslation();

  const logsMemo = React.useMemo(() => {
    return rawLogs.map((log) => {
      const line = string2LogItem(log.line);
      return line;
    });
  }, [rawLogs]);

  const filteredLogs = React.useMemo(() => {
    if (!search) {
      return logsMemo;
    }
    return logsMemo.filter((log) => log.message.includes(search));
  }, [logsMemo, search]);

  const listRef = React.useRef<VirtualList<ILogViewerLogItemProps[]>>(null);
  const sizeMap = React.useRef<Record<number, number>>({});
  const setSize = React.useCallback((index: number, size: number) => {
    sizeMap.current = { ...sizeMap.current, [index]: size };
    listRef.current?.resetAfterIndex(index);
  }, []);
  const getSize = (index: number) => sizeMap.current[index] || 50;

  // const scrollToBottomCallback = React.useCallback(() => {
  //   listRef.current?.scrollToItem(filteredLogs.length - 1);
  // }, []);

  React.useEffect(() => {
    setTimeout(() => {
      listRef.current?.scrollToItem(filteredLogs.length - 1, "end");
    }, 0);
  }, [filteredLogs, prefix]);

  if (!filteredLogs.length) {
    return (
      <div className="flex h-full w-full items-center justify-center">
        <span className="text-gray-500 dark:text-gray-400">
          {t("popup.logViewer.noLogs")}
        </span>
      </div>
    );
  }

  return (
    <>
      <AutoSizer key={`LogViewerLogItemList-${prefix}`}>
        {({ width, height }: { width: number; height: number }) => (
          <VirtualList
            ref={listRef}
            width={width}
            height={height}
            itemCount={filteredLogs.length}
            itemSize={getSize}
            itemData={filteredLogs}
          >
            {({ data, index, style }) => (
              <div style={style}>
                <VirtualListItem
                  data={data}
                  index={index}
                  setSize={setSize}
                  windowWidth={width}
                  search={search}
                />
              </div>
            )}
          </VirtualList>
        )}
      </AutoSizer>
    </>
  );
}
