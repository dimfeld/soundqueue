<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { Progress } from '$lib/components/ui/progress';
  import { invoke, convertFileSrc } from '@tauri-apps/api/tauri';
  import { tick } from 'svelte';
  import { quadOut } from 'svelte/easing';
  import { tweened } from 'svelte/motion';

  interface Manifest {
    items: AudioFile[];
  }

  interface AudioFile {
    name: string;
    directions: string;
    path: string;
    start_at?: number;
    stop_at?: number;
    fade_in?: number;
    fade_out?: number;
    volume?: number;
    length: number;
  }

  let backendError = '';
  let manifest: Manifest | null;
  async function getManifest() {
    try {
      manifest = await invoke('get_manifest');
      console.dir(manifest);
      backendError = '';
    } catch (e) {
      backendError = e;
    }
  }
  getManifest();

  let volume = tweened(1, { delay: 0, duration: 0, easing: quadOut });
  let currentIndex = 0;
  $: currentFile = manifest?.items[currentIndex];
  $: currentFileSrc = currentFile ? convertFileSrc(currentFile?.path) : '';
  $: if (currentFile && audioEl && audioEl.src != currentFileSrc) {
    console.log(`Current is ${currentFile.name}`);
    audioEl.src = convertFileSrc(currentFile.path);

    if (currentFile.fade_in) {
      volume.set(0);
    } else {
      volume.set(currentFile.volume ?? 1);
    }
  }

  let audioEl: HTMLAudioElement;

  async function playAudio(from = 0) {
    if (!audioEl || !currentFile) {
      return;
    }

    desiredState = 'playing';

    console.log('playAudio');

    let startAt = (currentFile.start_at ?? 0) + from;
    audioEl.currentTime = startAt;
    audioEl.play();

    if (currentFile.fade_in) {
      volume.set(currentFile.volume ?? 1, { duration: currentFile.fade_in });
    }
  }

  function finishedPlaying() {
    console.log('finishedPlaying');
    desiredState = 'stopped';
    audioEl.pause();
    setCurrent(currentIndex + 1);
  }

  let desiredState: 'stopped' | 'playing' | 'paused' = 'stopped';
  $: console.log('moving to', desiredState);

  function stop() {
    audioEl.pause();
    audioPos = 0;
    desiredState = 'stopped';
  }

  function resume() {
    if (!currentFile) {
      return;
    }
    audioEl.play();
    volume.set(currentFile.volume ?? 1);
    desiredState = 'playing';
  }

  function pause() {
    desiredState = 'paused';
    if (currentFile?.fade_out) {
      volume.set(0, { duration: currentFile.fade_out });
      setTimeout(() => audioEl.pause(), currentFile.fade_out);
    } else {
      audioEl.pause();
    }
  }

  function togglePause() {
    console.log('togglePause');
    if (audioEl.paused) {
      if (desiredState === 'stopped') {
        playAudio();
      } else {
        resume();
      }
    } else {
      pause();
    }
  }

  function setCurrent(newIndex: number) {
    if (!manifest) {
      return;
    }

    desiredState = 'stopped';
    audioEl.pause();
    audioPos = 0;
    return tick().then(() => {
      cards[newIndex]?.scrollIntoView({ behavior: 'smooth' });
      currentIndex = Math.max(0, Math.min(newIndex, manifest.items.length - 1));
      console.log(`current is ${newIndex}`);
    });
  }

  function globalKey(e: KeyboardEvent) {
    if (e.code === 'ArrowUp') {
      e.preventDefault();
      setCurrent(currentIndex - 1);
    } else if (e.code === 'ArrowDown') {
      e.preventDefault();
      setCurrent(currentIndex + 1);
    } else if (e.code === 'Space') {
      togglePause();
      e.preventDefault();
    }
  }

  let audioPos = 0;
  let paused = true;
  let audioDuration = 0;

  $: adjustedCurrent = Math.max(0, audioPos - (currentFile?.start_at ?? 0));
  $: maxDuration = Math.max(
    0,
    Math.min(audioDuration, currentFile?.stop_at ?? Infinity) - (currentFile?.start_at ?? 0)
  );

  $: if (audioEl && currentFile?.stop_at && !audioEl.paused && audioPos >= currentFile.stop_at) {
    finishedPlaying();
  }

  $: console.dir({
    currentIndex,
    currentFile,
    desiredState,
    audioDuration,
    paused,
    maxDuration,
  });

  let cards: HTMLDivElement[] = [];
</script>

<svelte:window on:keydown={globalKey} />

<main class="flex h-screen flex-col gap-4 p-2">
  {#if backendError}
    <p>{backendError}</p>
  {/if}

  <div class="flex justify-between">
    <audio
      bind:this={audioEl}
      bind:duration={audioDuration}
      bind:currentTime={audioPos}
      bind:paused
      bind:volume={$volume}
      on:ended={finishedPlaying}
      controls
    ></audio>

    <div class="flex gap-2">
      <div class="flex flex-col tabular-nums">
        <span>
          Pos: {adjustedCurrent.toFixed(1)} / {maxDuration.toFixed(2)}
        </span>
        <span>Volume: {Math.round($volume * 100)} %</span>
      </div>
      <Button on:click={() => getManifest()}>Reload</Button>
    </div>
  </div>

  <Card.Card>
    <Card.CardHeader>
      <Card.CardTitle>{currentFile?.name}</Card.CardTitle>
    </Card.CardHeader>
    <Card.CardContent class="flex flex-col gap-2">
      <div class="flex gap-2">
        <Button variant="default" on:click={() => playAudio()}>Play</Button>
        <Button variant="default" on:click={() => togglePause()}
          >{#if paused}Resume{:else}Pause{/if}</Button
        >
        <Button variant="default" on:click={stop}>Stop</Button>
      </div>
      <Progress value={adjustedCurrent} max={maxDuration} playing={!paused} class="w-full" />
    </Card.CardContent>
  </Card.Card>

  <div class="flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto">
    {#each manifest?.items ?? [] as file, i}
      <Card.Card
        bind:element={cards[i]}
        class={i == currentIndex ? 'bg-gray-200 dark:bg-gray-800' : ''}
      >
        <Card.CardHeader>
          <Card.CardTitle class="text-base font-medium">{file.name}</Card.CardTitle>
        </Card.CardHeader>
        <Card.CardContent>
          {#if file.directions}
            <p>{file.directions}</p>
          {/if}
          <div class="flex gap-2">
            <Button
              variant="default"
              on:click={() => {
                setCurrent(i);
              }}>Select</Button
            >
            <Button
              variant="default"
              on:click={async () => {
                await setCurrent(i);
                tick().then(() => playAudio());
              }}>Play</Button
            >
          </div>
        </Card.CardContent>
      </Card.Card>
    {/each}
  </div>
</main>
