import { BabyAgent } from './types';
import { TREDO_CONFIG } from './Tredo';
import { RISK_CONFIG } from './Risk';
import { NEWS_CONFIG } from './News';
import { TANTRA_CONFIG } from './Tantra';
import { BACKTESTER_CONFIG } from './Backtester';
import { PORTFOLIO_CONFIG } from './Portfolio';
import { ARBITRAGE_CONFIG } from './Arbitrage';
import { COMPLIANCE_CONFIG } from './Compliance';

export const DEFAULT_BABY_AGENTS: BabyAgent[] = [
  TREDO_CONFIG,
  RISK_CONFIG,
  NEWS_CONFIG,
  TANTRA_CONFIG,
  BACKTESTER_CONFIG,
  PORTFOLIO_CONFIG,
  ARBITRAGE_CONFIG,
  COMPLIANCE_CONFIG,
];
export type { BabyAgent };
