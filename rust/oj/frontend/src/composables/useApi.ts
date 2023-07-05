import { useFetch, MaybeRef, UseFetchOptions } from '@vueuse/core';
import { computed, unref } from 'vue';

const API_ROOT = 'http://localhost:12345';

export function apiUrl(path: MaybeRef<string>) {
  return computed(() => `${API_ROOT}${unref(path)}`);
}

// A wrapper of <https://vueuse.org/core/useFetch> that accepts the API path instead of full URL.
export function useApi(path: MaybeRef<string>, options?: UseFetchOptions) {
  if (options) {
    return useFetch(apiUrl(path), options).json<any>();
  }
  return useFetch(apiUrl(path)).json<any>();
}
