import { writable } from 'svelte/store';

export interface AppError {
  code: string;
  message: string;
  details?: string;
  timestamp: string;
}

export interface ErrorState {
  current_error: AppError | null;
  history: AppError[];
  is_visible: boolean;
}

function createErrorStore() {
  const { subscribe, set, update } = writable<ErrorState>({
    current_error: null,
    history: [],
    is_visible: false,
  });

  return {
    subscribe,
    addError: (error: AppError) => {
      update(state => ({
        ...state,
        current_error: error,
        history: [...state.history.slice(-19), error], // Keep last 20
        is_visible: true,
      }));

      // Auto-hide after 5 seconds
      const timer = setTimeout(() => {
        update(state => ({ ...state, is_visible: false }));
      }, 5000);

      // Store timer for cleanup if needed
      return () => clearTimeout(timer);
    },

    clearError: () => {
      update(state => ({ ...state, current_error: null, is_visible: false }));
    },

    clearHistory: () => {
      update(state => ({ ...state, history: [] }));
    },
  };
}

export const errorStore = createErrorStore();
