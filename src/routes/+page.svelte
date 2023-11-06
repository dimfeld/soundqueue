<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { Progress } from '$lib/components/ui/progress';
  import { invoke, convertFileSrc } from '@tauri-apps/api/tauri';
  import { onMount, tick } from 'svelte';

  interface Manifest {
    items: AudioFile[];
  }

  interface AudioFile {
    name: string;
    directions: string;
    path: string;
    startAt?: number;
    stopAt?: number;
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

  let currentIndex = 0;
  $: currentFile = manifest?.items[currentIndex];

  let audioEl: HTMLAudioElement;

  async function playAudio(from = 0) {
    if (!audioEl || !currentFile) {
      return;
    }

    audioEl.src = convertFileSrc(currentFile.path);
    let startAt = (currentFile.startAt ?? 0) + from;
    audioEl.currentTime = startAt;
    audioEl.volume = currentFile.volume ?? 1;
    audioEl.play();
  }

  // TODO hook this up
  function finishedPlaying() {
    if (!manifest) {
      return;
    }

    if (currentIndex < manifest.files.length - 1) {
      currentIndex++;
      audioPos = 0;
    }
  }

  function stop() {
    audioEl.pause();
    audioPos = 0;
  }

  function resume() {
    audioEl.play();
  }

  function pause() {
    audioEl.pause();
  }

  let audioPos = 0;
  let paused = true;
  let audioDuration = 0;
</script>

<main class="p-2">
  {#if backendError}
    <p>{backendError}</p>
  {/if}

  <audio
    bind:this={audioEl}
    bind:duration={audioDuration}
    bind:currentTime={audioPos}
    bind:paused
    controls
  ></audio>

  <div class="flex gap-2">
    <Button variant="default" on:click={() => playAudio()}>Play</Button>
    <Button
      variant="default"
      on:click={() => {
        if (paused) {
          resume();
        } else {
          pause();
        }
      }}
      >{#if paused}Resume{:else}Pause{/if}</Button
    >
    <Button variant="default" on:click={stop}>Stop</Button>
  </div>
  <Progress value={audioPos} max={audioDuration} class="w-full" />

  {#each manifest?.items ?? [] as file, i}
    <Card.Card class={i == currentIndex ? 'bg-gray-200' : ''}>
      <Card.CardHeader>
        <Card.CardTitle>{file.name}</Card.CardTitle>
      </Card.CardHeader>
      <Card.CardContent>
        {#if file.directions}
          <p>{file.directions}</p>
        {/if}
        <div class="flex gap-2">
          <Button
            variant="default"
            on:click={() => {
              currentIndex = i;
              tick().then(() => playAudio());
            }}>Play</Button
          >
          {#if currentIndex == i}
            <Button
              variant="default"
              on:click={() => {
                if (paused) {
                  resume();
                } else {
                  pause();
                }
              }}
              >{#if paused}Resume{:else}Pause{/if}</Button
            >
            <Button variant="default" on:click={stop}>Stop</Button>
          {/if}
        </div>
        <Progress value={i == currentIndex ? audioPos : 0} max={file.length} class="w-full" />
      </Card.CardContent>
    </Card.Card>
  {/each}
</main>
