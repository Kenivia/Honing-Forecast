export function get_readable(
  track: MediaStreamVideoTrack,
): ReadableStream<VideoFrame> {
  if (typeof MediaStreamTrackProcessor !== "undefined") {
    return new MediaStreamTrackProcessor({ track }).readable;
  }

  // no clue what's going on here on the firefox path but seems to work (and performance is really not bad)
  return createCanvasFrameReadable(track);
}

function driveVideoFrames(
  video: HTMLVideoElement,
  onFrame: (now: number, mediaTimeSec: number) => void,
): () => void {
  let stopped = false;
  const supportsRVFC =
    "requestVideoFrameCallback" in HTMLVideoElement.prototype;
  const tick = supportsRVFC
    ? (now: number, meta: VideoFrameCallbackMetadata) => {
        if (stopped) return;
        onFrame(now, meta.mediaTime);
        video.requestVideoFrameCallback(tick as any);
      }
    : (now: number) => {
        if (stopped) return;
        onFrame(now, video.currentTime);
        requestAnimationFrame(tick as any);
      };
  supportsRVFC
    ? video.requestVideoFrameCallback(tick as any)
    : requestAnimationFrame(tick as any);
  return () => {
    stopped = true;
  };
}

async function makeSourceVideo(
  track: MediaStreamVideoTrack,
): Promise<HTMLVideoElement> {
  const video = document.createElement("video");
  video.srcObject = new MediaStream([track]);
  video.muted = true;
  video.playsInline = true;
  await video.play();
  return video;
}

function createCanvasFrameReadable(
  track: MediaStreamVideoTrack,
): ReadableStream<VideoFrame> {
  let video: HTMLVideoElement;
  let stopDriving: () => void;

  return new ReadableStream<VideoFrame>({
    async start(controller) {
      video = await makeSourceVideo(track);
      stopDriving = driveVideoFrames(video, async (_now, mediaTimeSec) => {
        const bitmap = await createImageBitmap(video); // cheap, GPU-backed on most browsers
        controller.enqueue(
          new VideoFrame(bitmap, { timestamp: mediaTimeSec * 1e6 }),
        );
        // Note: VideoFrame(bitmap,...) does NOT close the bitmap for you.
        bitmap.close();
      });
    },
    cancel() {
      stopDriving?.();
      video?.pause();
      track.stop();
    },
  });
}
