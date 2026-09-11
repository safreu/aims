import { configureSync, getConsoleSink } from "@logtape/logtape";
import { diagnosticSink } from "./diagnosticsSink";

configureSync({
  sinks: {
    console: getConsoleSink(),
    diagnostics: diagnosticSink,
  },

  loggers: [
    {
      category: ["aims"],
      lowestLevel: "debug",
      sinks: ["console", "diagnostics"],
    },
  ],
});
