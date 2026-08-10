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
  let video: HTMLVideoElement | null = null;
  let stopDriving: (() => void) | undefined;
  let capturing = false;

  return new ReadableStream<VideoFrame>({
    async start(controller) {
      video = await makeSourceVideo(track);

      stopDriving = driveVideoFrames(video, (_now, mediaTimeSec) => {
        // Backpressure: don't produce frames the consumer hasn't asked for
        if (controller.desiredSize !== null && controller.desiredSize <= 0) {
          return;
        }

        if (capturing || !video) return;
        capturing = true;

        try {
          // Optimization: VideoFrame can consume an HTMLVideoElement directly.
          // This entirely bypasses the need for createImageBitmap.
          const frame = new VideoFrame(video, {
            timestamp: mediaTimeSec * 1e6,
          });
          controller.enqueue(frame);
        } catch (err) {
          console.warn("Failed to capture VideoFrame:", err);
        } finally {
          capturing = false;
        }
      });
    },
    cancel() {
      stopDriving?.();
      if (video) {
        video.pause();
        // Crucial for GC: Detach the stream from the video element
        video.srcObject = null;
        video.removeAttribute("src");
        video.load();
        video = null;
      }
      track.stop();
    },
  });
}
