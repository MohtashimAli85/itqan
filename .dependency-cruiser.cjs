/** @type {import('dependency-cruiser').IConfiguration} */
module.exports = {
  forbidden: [
    {
      name: "no-cross-module-imports",
      comment:
        "A module imports only src/core and src/shared, never another module (PLAN 16, C.4).",
      severity: "error",
      from: { path: "^src/modules/([^/]+)/" },
      to: { path: "^src/modules/", pathNot: "^src/modules/$1/" },
    },
    {
      name: "modules-use-the-core-surface",
      comment:
        "Modules use core only through src/core/registry and src/core/ui, never shell internals.",
      severity: "error",
      from: { path: "^src/modules/" },
      to: { path: "^src/core/", pathNot: "^src/core/(registry|ui)/" },
    },
    {
      name: "no-circular",
      comment: "Import cycles break module initialisation order.",
      severity: "error",
      from: {},
      to: { circular: true },
    },
    {
      name: "core-imports-only-the-registry",
      comment: "Core reaches modules only through src/modules/index.ts.",
      severity: "error",
      from: { path: "^src/core/" },
      to: { path: "^src/modules/", pathNot: "^src/modules/index\\.ts$" },
    },
    {
      name: "registry-imports-only-manifests",
      comment: "The registry lists module manifests and nothing deeper.",
      severity: "error",
      from: { path: "^src/modules/index\\.ts$" },
      to: {
        path: "^src/modules/[^/]+/",
        pathNot: "^src/modules/[^/]+/index\\.ts$",
      },
    },
    {
      name: "shared-stays-independent",
      comment: "src/shared imports neither core nor modules.",
      severity: "error",
      from: { path: "^src/shared/" },
      to: { path: "^src/(core|modules)/" },
    },
  ],
  options: {
    doNotFollow: { path: "node_modules" },
    exclude: { path: "^src/shared/bindings/" },
    tsPreCompilationDeps: true,
    tsConfig: { fileName: "tsconfig.json" },
    enhancedResolveOptions: {
      exportsFields: ["exports"],
      conditionNames: ["import", "require", "node", "default", "types"],
      extensions: [".ts", ".tsx", ".js", ".d.ts"],
    },
  },
};
