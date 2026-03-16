/**
 * @deprecated Import from ./tool-displays/ directory instead.
 *
 * This file re-exports from the split modules for backward compatibility.
 * New code should import directly:
 *   import { ContentSection } from "./tool-displays/primitives";
 *   import { ReadToolDisplay } from "./tool-displays/tool-renderers";
 */
export {
  ContentSection,
  TokenFooter,
  EmptyResult,
  ReadToolDisplay,
  WriteToolDisplay,
  BashToolDisplay,
  EditToolDisplay,
  TaskCreateDisplay,
  GenericToolInput,
  parseSerenaResult,
  SerenaResultDisplay,
  stripLineNumbers,
} from "./tool-displays/index";
