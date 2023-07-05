import { MaybeRef, useTitle as vueuseTitle } from '@vueuse/core';
import { computed, unref } from 'vue';
import { RouteRecordName } from 'vue-router';

const OJ_NAME = 'Rust Course Online Judge';

// Set the title of the page.
export default function useTitle(title: MaybeRef<RouteRecordName | null | undefined>) {
  return vueuseTitle(
    computed(
      () => {
        const t = unref(title);
        if (typeof t === 'string') {
          return `${t} - ${OJ_NAME}`;
        }
        return OJ_NAME;
      },
    ),
  );
}
