<script lang="ts">
  import type { HTMLAttributes } from 'svelte/elements';
  import { cn } from '$lib/utils';

  type $$Props = HTMLAttributes<HTMLDivElement> & { element?: HTMLDivElement };

  let className: $$Props['class'] = undefined;
  export { className as class };
  export let element: HTMLDivElement | undefined = undefined;
</script>

<div
  bind:this={element}
  class={cn('rounded-lg border bg-card text-card-foreground shadow-sm', className)}
  {...$$restProps}
>
  <slot />
</div>
