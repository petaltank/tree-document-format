/** A single validation diagnostic (error, warning, or advisory). */
export interface Diagnostic {
  rule: string;
  message: string;
  location: string;
  severity: "error" | "warning" | "advisory";
}

/** Aggregate statistics about a document. */
export interface ValidationStats {
  nodeCount: number;
  edgeCount: number;
  trunkLength: number;
  branchCount: number;
  tier: number;
}

/** Result of `validate()`. */
export interface ValidateResult {
  isValid: boolean;
  errors: Diagnostic[];
  warnings: Diagnostic[];
  advisories: Diagnostic[];
  stats: ValidationStats;
  /** Present only when the document cannot be parsed at all. */
  error?: string;
}

/** A single step along the trunk path. */
export interface TrunkStep {
  nodeId: string;
  content: string;
  branchCount: number;
  branchLabels: string[];
  isTerminal: boolean;
  trunkTarget: string | null;
}

/** Result of `view()`. */
export interface ViewResult {
  title: string;
  stats: string;
  steps: TrunkStep[];
  /** Present only on error. */
  error?: string;
}

/** Result of `info()`. */
export interface InfoResult {
  nodeCount: number;
  edgeCount: number;
  trunkLength: number;
  branchCount: number;
  tier: number;
  isValid: boolean;
  /** Present only on error. */
  error?: string;
}

/** Validate a `.tree.json` document string. */
export function validate(json_str: string): ValidateResult;

/** Build a trunk-path view of a `.tree.json` document string. */
export function view(json_str: string): ViewResult;

/** Get summary info for a `.tree.json` document string. */
export function info(json_str: string): InfoResult;
