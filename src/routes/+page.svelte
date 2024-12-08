<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { Progress } from '$lib/components/ui/progress';
  import { convertFileSrc, invoke } from '@tauri-apps/api/core';
  import { Slider } from '$lib/components/ui/slider';
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
    actions?: {
      name: string;
      volume: number;
      duration?: number;
    }[];
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
  $: if (audioEl) audioEl.volume = $volume;
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

    volume.set(currentFile.volume ?? 1, { duration: currentFile.fade_in ?? 0, easing: quadOut });
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

  async function pause(allowFadeOut: boolean): Promise<void> {
    desiredState = 'paused';
    const fade_out = currentFile?.fade_out;
    if (allowFadeOut && fade_out) {
      volume.set(0, { duration: fade_out, easing: quadOut });
      await new Promise<void>((resolve) => {
        setTimeout(() => {
          audioEl.pause();
          resolve();
        }, fade_out);
      });
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
      pause(false);
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

  async function stopAndGotoNext() {
    let nextIndex = currentIndex + 1;
    if (desiredState === 'playing') {
      await pause(true);
    }
    await setCurrent(nextIndex);
  }

  function globalKey(e: KeyboardEvent) {
    if (e.code === 'ArrowUp') {
      e.preventDefault();
      setCurrent(currentIndex - 1);
    } else if (e.code === 'ArrowDown') {
      e.preventDefault();
      setCurrent(currentIndex + 1);
    } else if (e.key === 'p') {
      e.preventDefault();
      togglePause();
    } else if (e.code === 'Space') {
      if (desiredState === 'playing') {
        stopAndGotoNext();
      } else {
        togglePause();
      }
      e.preventDefault();
    }
  }

  let audioPos = 0;
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
    maxDuration,
  });

  function setVolume(newVolume: number, duration?: number) {
    volume.set(newVolume, {
      duration: duration ? duration * 1000 : 0,
      easing: quadOut
    });
  }

  let cards: HTMLDivElement[] = [];
</script>

<svelte:window on:keydown={globalKey} />

<main class="flex h-screen flex-col gap-4 p-2">
  {#if backendError}
    <p>{backendError}</p>
  {/if}

  <Card.Card class="bg-gray-200/20 dark:bg-gray-800/20">
    <Card.CardHeader class="grid w-full grid-cols-[1fr_auto] ">
      <div class="grid grid-cols-1 gap-4">
        <Card.CardTitle>{currentFile?.name}</Card.CardTitle>
        <audio
          bind:this={audioEl}
          bind:duration={audioDuration}
          bind:currentTime={audioPos}
          on:ended={finishedPlaying}
          controls
        ></audio>
      </div>
      <div class="flex gap-4">
        <div class="flex flex-col tabular-nums">
          <span>
            Pos: {adjustedCurrent.toFixed(1)} / {maxDuration.toFixed(2)}
          </span>
        </div>
        <Button size="sm" variant="secondary" on:click={() => getManifest()}>Reload</Button>
      </div>
    </Card.CardHeader>
    <Card.CardContent class="flex flex-col gap-2">
      <div class="flex gap-2">
        <Button variant="default" on:click={() => stopAndGotoNext()}>Finish</Button>
        <Button variant="default" class="w-20" on:click={() => togglePause()}>
          {#if desiredState === 'playing'}Pause{:else}Resume{/if}
        </Button>
        <Button variant="default" on:click={() => playAudio()}>Play</Button>
        <Button variant="default" on:click={stop}>Stop</Button>
        {#if currentFile}
          <div class="ml-auto flex items-center gap-4 pb-2">
            <Slider
              class="w-48"
              value={[currentFile.volume ?? 1]}
              onValueChange={(vol) => {
                if (currentFile) {
                  currentFile.volume = vol[0];
                  setVolume(vol[0]);
                }
              }}
              min={0}
              max={1}
              step={0.01}
            />
            <span>{Math.round($volume * 100)}%</span>
          </div>
        {/if}
      </div>
      <Progress
        value={adjustedCurrent}
        max={maxDuration}
        playing={desiredState === 'playing'}
        class="w-full"
      />
    </Card.CardContent>
  </Card.Card>

  <div class="flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto">
    {#each manifest?.items ?? [] as file, i}
      <Card.Card
        bind:element={cards[i]}
        class={i == currentIndex ? 'bg-orange-200/20 dark:bg-orange-800/20' : ''}
      >
        <Card.CardHeader>
          <Card.CardTitle class="text-base font-medium">{file.name}</Card.CardTitle>
        </Card.CardHeader>
        <Card.CardContent>
          {#if file.directions}
            <p>{file.directions}</p>
          {/if}
          <div class="flex items-center gap-2">
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
            {#if currentIndex === i && file.actions && file.actions.length > 0}
              <div class="flex-1 flex justify-end gap-2">
                {#each file.actions as action}
                  <Button
                    variant="secondary"
                    on:click={async () => {
                      await setCurrent(i);
                      setVolume(action.volume, action.duration);
                    }}
                  >
                    {action.name}
                  </Button>
                {/each}
              </div>
            {/if}
          </div>
        </Card.CardContent>
      </Card.Card>
    {/each}
  </div>
</main>
