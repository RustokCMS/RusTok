// Admin modules register their nav through module-owned package entrypoints.
// Host shell code should not import business UI feature folders directly.
import '../../packages/blog/src';
import '../../packages/rustok-product/src';
import '../../packages/workflow/src';

export type { AdminModule } from './types';
export {
  registerAdminModule,
  getAdminModules,
  getAdminNavItems
} from './registry';
