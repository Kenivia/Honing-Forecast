/* eslint-disable no-undef */
import { markRaw } from "vue";

// Owns the capture stream, and so the user's permission. Lives in roster_config, never persisted.
export function create_frame_source() {
  let stream: MediaStream | null = null;
  let reader: ReadableStreamDefaultReader<VideoFrame> | null = null;
  let cleanup: (() => void) | null = null;
  let size = { width: 0, height: 0 };

  function stop() {
    reader?.cancel();
    stream?.getTracks().forEach((t) => t.stop());
    cleanup?.();
    reader = null;
    stream = null;
    cleanup = null;
  }

  return markRaw({
    start(
      new_stream: MediaStream,
      width: number,
      height: number,
      new_cleanup: (() => void) | null = null,
    ) {
      stop();
      stream = new_stream;
      size = { width, height };
      cleanup = new_cleanup;
      reader = get_readable(
        stream.getVideoTracks()[0] as MediaStreamVideoTrack,
      ).getReader();
    },
    stop,
    // undefined once the stream has ended; the caller closes or transfers the frame
    read: async () => (await reader?.read())?.value,
    get stream() {
      return stream;
    },
    get size() {
      return size;
    },
  });
}

function get_readable(
  track: MediaStreamVideoTrack,
): ReadableStream<VideoFrame> {
  if (typeof MediaStreamTrackProcessor !== "undefined") {
    return new MediaStreamTrackProcessor({ track, maxBufferSize: FRAME_BUFFER })
      .readable;
  }

  return createCanvasFrameReadable(track);
}

// Frames a second asked of the screen share. A tooltip can be readable for only a frame or two
// of a short hover, so every frame of the capture is scanned and this is not lowered lightly.
export const CAPTURE_FPS = 30;
// frames kept while a scan runs long (an anchor search takes several frames' worth)
const FRAME_BUFFER = 8;

// Image / video file as a canvas track. Videos hold their last frame once finished.
export async function file_to_stream(file: File) {
  const canvas = document.createElement("canvas");
  let source: HTMLVideoElement | ImageBitmap;
  let url: string | null = null;

  if (file.type.startsWith("video")) {
    url = URL.createObjectURL(file);
    const video = document.createElement("video");
    video.src = url;
    video.muted = true;
    video.playsInline = true;
    await video.play();
    canvas.width = video.videoWidth;
    canvas.height = video.videoHeight;
    source = video;
  } else {
    source = await createImageBitmap(file);
    canvas.width = source.width;
    canvas.height = source.height;
  }

  const ctx = canvas.getContext("2d");
  const draw = () => ctx.drawImage(source, 0, 0);
  draw();
  // a video is drawn frame by frame as it plays; the timer keeps frames coming once it has ended
  const video = source instanceof HTMLVideoElement ? source : null;
  const stop_driving = video ? driveVideoFrames(video, draw) : null;
  const timer = setInterval(draw, video ? 500 : 1000 / CAPTURE_FPS);

  return {
    stream: canvas.captureStream(),
    width: canvas.width,
    height: canvas.height,
    cleanup: () => {
      clearInterval(timer);
      stop_driving?.();
      if (source instanceof HTMLVideoElement) {
        source.pause();
        source.removeAttribute("src");
        source.load();
        URL.revokeObjectURL(url);
      } else {
        source.close();
      }
    },
  };
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

  return new ReadableStream<VideoFrame>(
    {
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
    },
    { highWaterMark: FRAME_BUFFER },
  );
}
