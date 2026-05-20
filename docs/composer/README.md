# Composer Documentation

This directory contains documentation and extracted resources related to Cursor's **Composer** feature.

## Contents

- [protos.md](file:///Users/jaredboynton/__devlocal/unified-model-proxy-v2/docs/composer/protos.md): Extracted Protobuf definitions for the `BackgroundComposerService` and related message types.

## Source

The definitions were extracted from the compiled JavaScript bundles of the Cursor application:
`/Applications/Cursor.app/Contents/Resources/app/out/vs/workbench/workbench.desktop.main.js`

Extraction was performed by searching for `aiserver.v1` type definitions and reconstructing the `.proto` structure based on the field descriptors used by the `@bufbuild/protobuf` (or compatible) runtime.
