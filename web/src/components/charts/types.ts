/**
 * Types for chart components.
 */

export interface BarChartProps {
  data: [string, number][];
  maxItems?: number;
  accentColor?: string;
}

export interface DayChartProps {
  data: number[];
  label?: string;
}

export interface TokenBreakdownBarProps {
  input: number;
  output: number;
  cacheRead: number;
  cacheCreation: number;
}
