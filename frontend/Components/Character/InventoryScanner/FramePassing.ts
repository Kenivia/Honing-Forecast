/* eslint-disable no-undef */

export function get_readable(
  track: MediaStreamVideoTrack,
): ReadableStream<VideoFrame> {
  if (typeof MediaStreamTrackProcessor !== "undefined") {
    return new MediaStreamTrackProcessor({ track }).readable;
  }

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
  let stopDriving: (() => void) | undefined;
  let capturing = false; // prevents overlapping createImageBitmap calls

  return new ReadableStream<VideoFrame>({
    async start(controller) {
      video = await makeSourceVideo(track);

      stopDriving = driveVideoFrames(video, async (_now, mediaTimeSec) => {
        // Backpressure: don't produce frames the consumer hasn't asked for
        if (controller.desiredSize !== null && controller.desiredSize <= 0) {
          return;
        }
        // Don't start a new capture while one is still in flight
        if (capturing) return;
        capturing = true;
        try {
          const bitmap = await createImageBitmap(video);
          const frame = new VideoFrame(bitmap, {
            timestamp: mediaTimeSec * 1e6,
          });
          bitmap.close();
          controller.enqueue(frame);
        } finally {
          capturing = false;
        }
      });
    },
    cancel() {
      stopDriving?.();
      video?.pause();
      track.stop();
    },
  });
}
