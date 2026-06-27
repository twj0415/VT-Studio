use crate::domain::media::{FfmpegSidecarStatusDto, MediaProbeDto, SidecarBinaryStatusDto};
use crate::domain::scene::VideoSegmentDto;
use crate::security::secret_guard::redact_text;
use crate::services::storage_service::{FileAccessPolicy, FileBucket, StorageService};
use serde::Deserialize;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const FFMPEG_BINARY: &str = "ffmpeg.exe";
const FFPROBE_BINARY: &str = "ffprobe.exe";
const FFMPEG_NOT_FOUND: &str = "ffmpeg.not_found";
const FFMPEG_PROCESS_FAILED: &str = "ffmpeg.process_failed";
const FFMPEG_PROBE_FAILED: &str = "ffmpeg.probe_failed";
const FFMPEG_INVALID_MEDIA: &str = "ffmpeg.invalid_media";
const FFMPEG_CONCAT_FAILED: &str = "ffmpeg.concat_failed";
const FFMPEG_TRANSCODE_FAILED: &str = "ffmpeg.transcode_failed";
const FFMPEG_BGM_MIX_FAILED: &str = "ffmpeg.bgm_mix_failed";
const FFMPEG_SUBTITLE_BURN_FAILED: &str = "ffmpeg.subtitle_burn_failed";
const STANDARD_VIDEO_CODEC: &str = "h264";
const STANDARD_ENCODER: &str = "libx264";
const STANDARD_PIXEL_FORMAT: &str = "yuv420p";
const STANDARD_FPS: u32 = 30;
const STANDARD_AUDIO_CODEC: &str = "aac";
const STANDARD_AUDIO_BITRATE: &str = "192k";
const STANDARD_SAMPLE_RATE: u32 = 44_100;
const FPS_EPSILON: f64 = 0.01;
const PROCESS_TEXT_MAX_LINES: usize = 200;
const PROCESS_TEXT_MAX_BYTES: usize = 32 * 1024;

trait VersionRunner {
    fn run_version(&self, binary_path: &Path) -> Result<String, String>;
}

trait ProbeRunner {
    fn run_probe(&self, ffprobe_path: &Path, media_path: &Path) -> Result<String, String>;
}

trait ConcatRunner {
    fn run_concat(
        &self,
        ffmpeg_path: &Path,
        filelist_path: &Path,
        output_path: &Path,
    ) -> Result<(), String>;
}

trait TranscodeRunner {
    fn run_transcode(
        &self,
        ffmpeg_path: &Path,
        input_path: &Path,
        output_path: &Path,
        spec: &TranscodeSpec,
        input_has_audio: bool,
    ) -> Result<(), String>;
}

trait BgmMixRunner {
    fn run_mix_bgm(
        &self,
        ffmpeg_path: &Path,
        video_path: &Path,
        bgm_path: &Path,
        output_path: &Path,
        options: &BgmMixOptions,
    ) -> Result<(), String>;
}

trait SubtitleBurnRunner {
    fn run_burn_subtitles(
        &self,
        ffmpeg_path: &Path,
        video_path: &Path,
        subtitle_path: &Path,
        output_path: &Path,
    ) -> Result<(), String>;
}

struct ProcessVersionRunner;
struct ProcessProbeRunner;
struct ProcessConcatRunner;
struct ProcessTranscodeRunner;
struct ProcessBgmMixRunner;
struct ProcessSubtitleBurnRunner;

impl VersionRunner for ProcessVersionRunner {
    fn run_version(&self, binary_path: &Path) -> Result<String, String> {
        let output = Command::new(binary_path)
            .arg("-version")
            .output()
            .map_err(|error| error.to_string())?;

        if !output.status.success() {
            let stderr = sanitize_process_message(&String::from_utf8_lossy(&output.stderr), &[]);
            return Err(if stderr.is_empty() {
                "process exited with a non-zero status.".to_string()
            } else {
                stderr
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let version = first_non_empty_line(&stdout)
            .or_else(|| first_non_empty_line(&stderr))
            .unwrap_or_else(|| "version output was empty.".to_string());
        Ok(version)
    }
}

impl ProbeRunner for ProcessProbeRunner {
    fn run_probe(&self, ffprobe_path: &Path, media_path: &Path) -> Result<String, String> {
        let output = Command::new(ffprobe_path)
            .args([
                "-v",
                "error",
                "-print_format",
                "json",
                "-show_format",
                "-show_streams",
            ])
            .arg(media_path)
            .output()
            .map_err(|error| error.to_string())?;

        if !output.status.success() {
            let stderr = sanitize_process_message(&String::from_utf8_lossy(&output.stderr), &[]);
            return Err(if stderr.is_empty() {
                "ffprobe exited with a non-zero status.".to_string()
            } else {
                stderr
            });
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

impl ConcatRunner for ProcessConcatRunner {
    fn run_concat(
        &self,
        ffmpeg_path: &Path,
        filelist_path: &Path,
        output_path: &Path,
    ) -> Result<(), String> {
        let output = Command::new(ffmpeg_path)
            .args(["-y", "-f", "concat", "-safe", "0", "-i"])
            .arg(filelist_path)
            .args(["-c", "copy"])
            .arg(output_path)
            .output()
            .map_err(|error| error.to_string())?;

        if output.status.success() {
            return Ok(());
        }

        let stderr = sanitize_process_message(&String::from_utf8_lossy(&output.stderr), &[]);
        Err(if stderr.is_empty() {
            "ffmpeg concat exited with a non-zero status.".to_string()
        } else {
            stderr
        })
    }
}

impl TranscodeRunner for ProcessTranscodeRunner {
    fn run_transcode(
        &self,
        ffmpeg_path: &Path,
        input_path: &Path,
        output_path: &Path,
        spec: &TranscodeSpec,
        input_has_audio: bool,
    ) -> Result<(), String> {
        let mut command = Command::new(ffmpeg_path);
        command.arg("-y").arg("-i").arg(input_path);
        if spec.include_audio && !input_has_audio {
            command.args(["-f", "lavfi", "-i"]).arg(format!(
                "anullsrc=channel_layout=stereo:sample_rate={}",
                spec.sample_rate
            ));
        }
        for arg in build_transcode_args(spec, input_has_audio) {
            command.arg(arg);
        }
        let output = command
            .arg(output_path)
            .output()
            .map_err(|error| error.to_string())?;

        if output.status.success() {
            return Ok(());
        }

        let stderr = sanitize_process_message(&String::from_utf8_lossy(&output.stderr), &[]);
        Err(if stderr.is_empty() {
            "ffmpeg transcode exited with a non-zero status.".to_string()
        } else {
            stderr
        })
    }
}

impl BgmMixRunner for ProcessBgmMixRunner {
    fn run_mix_bgm(
        &self,
        ffmpeg_path: &Path,
        video_path: &Path,
        bgm_path: &Path,
        output_path: &Path,
        options: &BgmMixOptions,
    ) -> Result<(), String> {
        let mut command = Command::new(ffmpeg_path);
        command.arg("-y").arg("-i").arg(video_path);
        if options.loop_bgm {
            command.args(["-stream_loop", "-1"]);
        }
        command.arg("-i").arg(bgm_path);
        for arg in build_bgm_mix_args(options) {
            command.arg(arg);
        }
        let output = command
            .arg(output_path)
            .output()
            .map_err(|error| error.to_string())?;

        if output.status.success() {
            return Ok(());
        }

        let stderr = sanitize_process_message(&String::from_utf8_lossy(&output.stderr), &[]);
        Err(if stderr.is_empty() {
            "ffmpeg bgm mix exited with a non-zero status.".to_string()
        } else {
            stderr
        })
    }
}

impl SubtitleBurnRunner for ProcessSubtitleBurnRunner {
    fn run_burn_subtitles(
        &self,
        ffmpeg_path: &Path,
        video_path: &Path,
        subtitle_path: &Path,
        output_path: &Path,
    ) -> Result<(), String> {
        let output = Command::new(ffmpeg_path)
            .arg("-y")
            .arg("-i")
            .arg(video_path)
            .args([
                "-vf",
                &format!("subtitles={}", escape_subtitle_filter_path(subtitle_path)),
            ])
            .args([
                "-c:v",
                STANDARD_ENCODER,
                "-preset",
                "veryfast",
                "-crf",
                "18",
            ])
            .args(["-c:a", "copy"])
            .args(["-movflags", "+faststart"])
            .arg(output_path)
            .output()
            .map_err(|error| error.to_string())?;

        if output.status.success() {
            return Ok(());
        }

        let stderr = sanitize_process_message(&String::from_utf8_lossy(&output.stderr), &[]);
        Err(if stderr.is_empty() {
            "ffmpeg subtitle burn exited with a non-zero status.".to_string()
        } else {
            stderr
        })
    }
}

pub fn check_ffmpeg_sidecars(workspace_root: &Path) -> Result<FfmpegSidecarStatusDto, String> {
    check_ffmpeg_sidecars_with_runner(workspace_root, &ProcessVersionRunner)
}

pub fn require_ffmpeg_sidecars(workspace_root: &Path) -> Result<FfmpegSidecarStatusDto, String> {
    let status = check_ffmpeg_sidecars(workspace_root)?;
    if status.ready {
        return Ok(status);
    }

    Err(format!(
        "{FFMPEG_NOT_FOUND}: {}",
        summarize_unready_sidecars(&status)
    ))
}

pub fn probe_media(
    workspace_root: &Path,
    relative_path: &str,
    media_kind: Option<&str>,
) -> Result<MediaProbeDto, String> {
    require_ffmpeg_sidecars(workspace_root)?;
    probe_media_with_runner(
        workspace_root,
        relative_path,
        media_kind,
        &ProcessProbeRunner,
    )
}

pub fn probe_video_segments(
    workspace_root: &Path,
    segments: &[VideoSegmentDto],
) -> Result<Vec<MediaProbeDto>, String> {
    segments
        .iter()
        .map(|segment| probe_media(workspace_root, &segment.video_path, Some("video")))
        .collect()
}

#[allow(dead_code)]
pub fn concat_segments(
    workspace_root: &Path,
    project_id: &str,
    task_id: &str,
    segment_paths: &[String],
) -> Result<String, String> {
    require_ffmpeg_sidecars(workspace_root)?;
    concat_segments_with_runner(
        workspace_root,
        project_id,
        task_id,
        segment_paths,
        &ProcessConcatRunner,
        &ProcessTranscodeRunner,
        None,
    )
}

pub fn concat_segments_with_probes(
    workspace_root: &Path,
    project_id: &str,
    task_id: &str,
    segment_paths: &[String],
    probes: &[MediaProbeDto],
) -> Result<String, String> {
    require_ffmpeg_sidecars(workspace_root)?;
    concat_segments_with_runner(
        workspace_root,
        project_id,
        task_id,
        segment_paths,
        &ProcessConcatRunner,
        &ProcessTranscodeRunner,
        Some(probes),
    )
}

#[derive(Debug, Clone)]
pub struct BgmMixOptions {
    pub bgm_relative_path: String,
    pub volume: f64,
    pub loop_bgm: bool,
    pub fade_in_seconds: f64,
    pub fade_out_seconds: f64,
    pub duration_seconds: f64,
    pub video_has_audio: bool,
}

pub fn mix_bgm_into_video(
    workspace_root: &Path,
    project_id: &str,
    task_id: &str,
    video_relative_path: &str,
    options: &BgmMixOptions,
) -> Result<String, String> {
    require_ffmpeg_sidecars(workspace_root)?;
    mix_bgm_into_video_with_runner(
        workspace_root,
        project_id,
        task_id,
        video_relative_path,
        options,
        &ProcessBgmMixRunner,
    )
}

pub fn burn_subtitles_into_video(
    workspace_root: &Path,
    project_id: &str,
    task_id: &str,
    video_relative_path: &str,
    subtitle_relative_path: &str,
) -> Result<String, String> {
    require_ffmpeg_sidecars(workspace_root)?;
    burn_subtitles_into_video_with_runner(
        workspace_root,
        project_id,
        task_id,
        video_relative_path,
        subtitle_relative_path,
        &ProcessSubtitleBurnRunner,
    )
}

fn check_ffmpeg_sidecars_with_runner(
    workspace_root: &Path,
    runner: &dyn VersionRunner,
) -> Result<FfmpegSidecarStatusDto, String> {
    let storage = StorageService::new(workspace_root);
    storage.initialize_workspace()?;
    let ffmpeg = check_sidecar_binary(&storage, FFMPEG_BINARY, runner);
    let ffprobe = check_sidecar_binary(&storage, FFPROBE_BINARY, runner);
    let ready = ffmpeg.executable && ffprobe.executable;

    Ok(FfmpegSidecarStatusDto {
        ffmpeg,
        ffprobe,
        ready,
        checked_at: current_timestamp_string(),
    })
}

fn mix_bgm_into_video_with_runner(
    workspace_root: &Path,
    project_id: &str,
    task_id: &str,
    video_relative_path: &str,
    options: &BgmMixOptions,
    runner: &dyn BgmMixRunner,
) -> Result<String, String> {
    validate_bgm_mix_options(options)?;
    let storage = StorageService::new(workspace_root);
    storage.initialize_workspace()?;
    let ffmpeg_path = storage
        .resolver()
        .resolve_existing_bucket_path(FileBucket::Sidecar, FFMPEG_BINARY)
        .map_err(|error| {
            format!(
                "{FFMPEG_NOT_FOUND}: {}",
                sanitize_sidecar_message(&error, None)
            )
        })?;
    let (video_bucket, video_bucket_relative_path) =
        split_workspace_relative_path(video_relative_path)?;
    let video_path = storage
        .resolver()
        .resolve_existing_bucket_path(video_bucket, video_bucket_relative_path)
        .map_err(|error| {
            format!(
                "{FFMPEG_INVALID_MEDIA}: {}",
                sanitize_probe_message(&error, None)
            )
        })?;
    let (bgm_bucket, bgm_bucket_relative_path) =
        split_workspace_relative_path(&options.bgm_relative_path)?;
    let bgm_path = storage
        .resolver()
        .resolve_existing_bucket_path(bgm_bucket, bgm_bucket_relative_path)
        .map_err(|error| {
            format!(
                "{FFMPEG_INVALID_MEDIA}: {}",
                sanitize_probe_message(&error, None)
            )
        })?;
    let output_relative_path = format!(
        "exports/{}/{}_final_bgm.mp4",
        sanitize_file_segment(project_id),
        sanitize_file_segment(task_id)
    );
    let output_path = storage
        .resolver()
        .resolve_bucket_path_for_write(FileBucket::Output, &output_relative_path)?;
    let log_replacements = vec![
        (workspace_root.to_path_buf(), "<workspace>".to_string()),
        (ffmpeg_path.clone(), sidecar_relative_path(FFMPEG_BINARY)),
        (video_path.clone(), video_relative_path.to_string()),
        (bgm_path.clone(), options.bgm_relative_path.clone()),
        (
            output_path.clone(),
            format!("outputs/{output_relative_path}"),
        ),
    ];
    let result = runner
        .run_mix_bgm(&ffmpeg_path, &video_path, &bgm_path, &output_path, options)
        .map_err(|error| {
            format!(
                "{FFMPEG_BGM_MIX_FAILED}: {}",
                sanitize_process_message(&error, &log_replacements)
            )
        });
    if let Err(error) = result {
        let _ = std::fs::remove_file(&output_path);
        return Err(error);
    }
    if !output_path.is_file() {
        return Err(
            "ffmpeg.bgm_output_missing: final BGM output file was not created.".to_string(),
        );
    }

    Ok(format!("outputs/{output_relative_path}"))
}

fn burn_subtitles_into_video_with_runner(
    workspace_root: &Path,
    project_id: &str,
    task_id: &str,
    video_relative_path: &str,
    subtitle_relative_path: &str,
    runner: &dyn SubtitleBurnRunner,
) -> Result<String, String> {
    validate_subtitle_path(subtitle_relative_path)?;
    let storage = StorageService::new(workspace_root);
    storage.initialize_workspace()?;
    let ffmpeg_path = storage
        .resolver()
        .resolve_existing_bucket_path(FileBucket::Sidecar, FFMPEG_BINARY)
        .map_err(|error| {
            format!(
                "{FFMPEG_NOT_FOUND}: {}",
                sanitize_sidecar_message(&error, None)
            )
        })?;
    let (video_bucket, video_bucket_relative_path) =
        split_workspace_relative_path(video_relative_path)?;
    let video_path = storage
        .resolver()
        .resolve_existing_bucket_path(video_bucket, video_bucket_relative_path)
        .map_err(|error| {
            format!(
                "{FFMPEG_INVALID_MEDIA}: {}",
                sanitize_probe_message(&error, None)
            )
        })?;
    let (subtitle_bucket, subtitle_bucket_relative_path) =
        split_workspace_relative_path(subtitle_relative_path)?;
    let subtitle_json_path = storage
        .resolver()
        .resolve_existing_bucket_path(subtitle_bucket, subtitle_bucket_relative_path)
        .map_err(|error| {
            format!(
                "{FFMPEG_INVALID_MEDIA}: {}",
                sanitize_probe_message(&error, None)
            )
        })?;
    let subtitle_srt_relative_path = format!(
        "composition/{}/{}_subtitles.srt",
        sanitize_file_segment(project_id),
        sanitize_file_segment(task_id)
    );
    let subtitle_srt_content = subtitle_json_to_srt(&subtitle_json_path)?;
    let subtitle_srt = storage.write_text(
        FileBucket::Temp,
        &subtitle_srt_relative_path,
        &subtitle_srt_content,
        FileAccessPolicy::TempOnly,
    )?;
    let output_relative_path = format!(
        "exports/{}/{}_final_subtitle.mp4",
        sanitize_file_segment(project_id),
        sanitize_file_segment(task_id)
    );
    let output_path = storage
        .resolver()
        .resolve_bucket_path_for_write(FileBucket::Output, &output_relative_path)?;
    let log_replacements = vec![
        (workspace_root.to_path_buf(), "<workspace>".to_string()),
        (ffmpeg_path.clone(), sidecar_relative_path(FFMPEG_BINARY)),
        (video_path.clone(), video_relative_path.to_string()),
        (
            subtitle_json_path.clone(),
            subtitle_relative_path.to_string(),
        ),
        (
            subtitle_srt.absolute_path.clone(),
            format!("temp/{subtitle_srt_relative_path}"),
        ),
        (
            output_path.clone(),
            format!("outputs/{output_relative_path}"),
        ),
    ];
    let result = runner
        .run_burn_subtitles(
            &ffmpeg_path,
            &video_path,
            &subtitle_srt.absolute_path,
            &output_path,
        )
        .map_err(|error| {
            format!(
                "{FFMPEG_SUBTITLE_BURN_FAILED}: {}",
                sanitize_process_message(&error, &log_replacements)
            )
        });
    let _ = std::fs::remove_file(&subtitle_srt.absolute_path);
    if let Err(error) = result {
        let _ = std::fs::remove_file(&output_path);
        return Err(error);
    }
    if !output_path.is_file() {
        return Err(
            "ffmpeg.subtitle_output_missing: final subtitle output file was not created."
                .to_string(),
        );
    }

    Ok(format!("outputs/{output_relative_path}"))
}

fn check_sidecar_binary(
    storage: &StorageService,
    binary_name: &str,
    runner: &dyn VersionRunner,
) -> SidecarBinaryStatusDto {
    let relative_path = sidecar_relative_path(binary_name);
    let resolved_path = storage
        .resolver()
        .resolve_existing_bucket_path(FileBucket::Sidecar, binary_name);

    let absolute_path = match resolved_path {
        Ok(path) => path,
        Err(error) => {
            return SidecarBinaryStatusDto {
                name: binary_name.to_string(),
                relative_path,
                exists: false,
                executable: false,
                version: None,
                error_code: Some(FFMPEG_NOT_FOUND.to_string()),
                message: Some(sanitize_sidecar_message(&error, None)),
            };
        }
    };

    let exists = absolute_path.is_file();
    if !exists {
        return SidecarBinaryStatusDto {
            name: binary_name.to_string(),
            relative_path,
            exists: false,
            executable: false,
            version: None,
            error_code: Some(FFMPEG_NOT_FOUND.to_string()),
            message: Some("sidecar file is missing.".to_string()),
        };
    }

    match runner.run_version(&absolute_path) {
        Ok(version) => SidecarBinaryStatusDto {
            name: binary_name.to_string(),
            relative_path,
            exists: true,
            executable: true,
            version: Some(sanitize_process_message(&version, &[])),
            error_code: None,
            message: None,
        },
        Err(error) => SidecarBinaryStatusDto {
            name: binary_name.to_string(),
            relative_path,
            exists: true,
            executable: false,
            version: None,
            error_code: Some(FFMPEG_PROCESS_FAILED.to_string()),
            message: Some(sanitize_sidecar_message(&error, Some(&absolute_path))),
        },
    }
}

fn probe_media_with_runner(
    workspace_root: &Path,
    relative_path: &str,
    media_kind: Option<&str>,
    runner: &dyn ProbeRunner,
) -> Result<MediaProbeDto, String> {
    let storage = StorageService::new(workspace_root);
    storage.initialize_workspace()?;
    let ffprobe_path = storage
        .resolver()
        .resolve_existing_bucket_path(FileBucket::Sidecar, FFPROBE_BINARY)
        .map_err(|error| {
            format!(
                "{FFMPEG_NOT_FOUND}: {}",
                sanitize_sidecar_message(&error, None)
            )
        })?;
    let (bucket, bucket_relative_path) = split_workspace_relative_path(relative_path)?;
    let media_path = storage
        .resolver()
        .resolve_existing_bucket_path(bucket, bucket_relative_path)
        .map_err(|error| {
            format!(
                "{FFMPEG_INVALID_MEDIA}: {}",
                sanitize_probe_message(&error, None)
            )
        })?;
    let output = runner
        .run_probe(&ffprobe_path, &media_path)
        .map_err(|error| {
            let replacements = vec![
                (workspace_root.to_path_buf(), "<workspace>".to_string()),
                (media_path.clone(), relative_path.to_string()),
            ];
            format!(
                "{FFMPEG_PROBE_FAILED}: {}",
                sanitize_process_message(&error, &replacements)
            )
        })?;
    parse_ffprobe_json(relative_path, media_kind.unwrap_or("unknown"), &output)
}

fn concat_segments_with_runner(
    workspace_root: &Path,
    project_id: &str,
    task_id: &str,
    segment_paths: &[String],
    runner: &dyn ConcatRunner,
    transcoder: &dyn TranscodeRunner,
    probes: Option<&[MediaProbeDto]>,
) -> Result<String, String> {
    if segment_paths.is_empty() {
        return Err(format!(
            "{FFMPEG_INVALID_MEDIA}: at least one selected video segment is required."
        ));
    }

    let storage = StorageService::new(workspace_root);
    storage.initialize_workspace()?;
    let ffmpeg_path = storage
        .resolver()
        .resolve_existing_bucket_path(FileBucket::Sidecar, FFMPEG_BINARY)
        .map_err(|error| {
            format!(
                "{FFMPEG_NOT_FOUND}: {}",
                sanitize_sidecar_message(&error, None)
            )
        })?;
    if let Some(probes) = probes {
        if probes.len() != segment_paths.len() {
            return Err(format!(
                "{FFMPEG_INVALID_MEDIA}: media probe count must match segment count."
            ));
        }
    }

    let mut segments = Vec::with_capacity(segment_paths.len());
    for relative_path in segment_paths {
        let (bucket, bucket_relative_path) = split_workspace_relative_path(relative_path)?;
        let absolute_path = storage
            .resolver()
            .resolve_existing_bucket_path(bucket, bucket_relative_path)
            .map_err(|error| {
                format!(
                    "{FFMPEG_INVALID_MEDIA}: {}",
                    sanitize_probe_message(&error, None)
                )
            })?;
        segments.push(ResolvedSegment {
            absolute_path,
            relative_path: relative_path.clone(),
        });
    }

    let transcode_spec = match probes {
        Some(probes) => build_transcode_spec(probes)?,
        None => None,
    };
    let concat_segment_paths = if let Some((spec, probes)) = transcode_spec.as_ref().zip(probes) {
        prepare_segments_for_concat(
            &storage,
            project_id,
            task_id,
            &segments,
            probes,
            spec,
            &ffmpeg_path,
            transcoder,
        )?
    } else {
        segments
            .iter()
            .map(|segment| segment.absolute_path.clone())
            .collect::<Vec<_>>()
    };

    let filelist_relative_path = format!(
        "composition/{}/{}_concat.txt",
        sanitize_file_segment(project_id),
        sanitize_file_segment(task_id)
    );
    let filelist_content = build_concat_filelist(&concat_segment_paths);
    let filelist = storage.write_text(
        FileBucket::Temp,
        &filelist_relative_path,
        &filelist_content,
        FileAccessPolicy::TempOnly,
    )?;
    let output_relative_path = format!(
        "exports/{}/{}_final.mp4",
        sanitize_file_segment(project_id),
        sanitize_file_segment(task_id)
    );
    let output_path = storage
        .resolver()
        .resolve_bucket_path_for_write(FileBucket::Output, &output_relative_path)?;
    let mut log_replacements = vec![
        (workspace_root.to_path_buf(), "<workspace>".to_string()),
        (ffmpeg_path.clone(), sidecar_relative_path(FFMPEG_BINARY)),
    ];
    log_replacements.extend(
        segments
            .iter()
            .map(|segment| (segment.absolute_path.clone(), segment.relative_path.clone()))
            .collect::<Vec<_>>(),
    );
    log_replacements.push((
        filelist.absolute_path.clone(),
        format!("temp/{}", filelist.relative_path),
    ));
    log_replacements.push((
        output_path.clone(),
        format!("outputs/{output_relative_path}"),
    ));

    let result = runner
        .run_concat(&ffmpeg_path, &filelist.absolute_path, &output_path)
        .map_err(|error| {
            format!(
                "{FFMPEG_CONCAT_FAILED}: {}",
                sanitize_process_message(&error, &log_replacements)
            )
        });
    let _ = std::fs::remove_file(&filelist.absolute_path);
    cleanup_transcoded_segments(&concat_segment_paths, &segments);
    result?;

    if !output_path.is_file() {
        return Err("ffmpeg.output_missing: final output file was not created.".to_string());
    }

    Ok(format!("outputs/{output_relative_path}"))
}

#[derive(Debug, Clone)]
struct ResolvedSegment {
    absolute_path: std::path::PathBuf,
    relative_path: String,
}

#[derive(Debug, Clone)]
struct TranscodeSpec {
    width: u32,
    height: u32,
    fps: u32,
    pixel_format: &'static str,
    encoder: &'static str,
    video_codec: &'static str,
    audio_codec: &'static str,
    audio_bitrate: &'static str,
    sample_rate: u32,
    include_audio: bool,
}

fn build_transcode_spec(probes: &[MediaProbeDto]) -> Result<Option<TranscodeSpec>, String> {
    if probes.is_empty() {
        return Ok(None);
    }

    let Some((width, height)) = probes
        .iter()
        .find_map(|probe| probe.width.zip(probe.height))
    else {
        return Err(format!(
            "{FFMPEG_INVALID_MEDIA}: video dimensions are required before transcoding."
        ));
    };

    Ok(Some(TranscodeSpec {
        width,
        height,
        fps: STANDARD_FPS,
        pixel_format: STANDARD_PIXEL_FORMAT,
        encoder: STANDARD_ENCODER,
        video_codec: STANDARD_VIDEO_CODEC,
        audio_codec: STANDARD_AUDIO_CODEC,
        audio_bitrate: STANDARD_AUDIO_BITRATE,
        sample_rate: STANDARD_SAMPLE_RATE,
        include_audio: probes.iter().any(|probe| probe.has_audio_stream),
    }))
}

fn prepare_segments_for_concat(
    storage: &StorageService,
    project_id: &str,
    task_id: &str,
    segments: &[ResolvedSegment],
    probes: &[MediaProbeDto],
    spec: &TranscodeSpec,
    ffmpeg_path: &Path,
    transcoder: &dyn TranscodeRunner,
) -> Result<Vec<std::path::PathBuf>, String> {
    let mut prepared_paths = Vec::with_capacity(segments.len());
    for (index, (segment, probe)) in segments.iter().zip(probes.iter()).enumerate() {
        if !requires_transcode(probe, spec) {
            prepared_paths.push(segment.absolute_path.clone());
            continue;
        }

        let relative_path = format!(
            "composition/{}/{}/segment_{:04}_normalized.mp4",
            sanitize_file_segment(project_id),
            sanitize_file_segment(task_id),
            index + 1
        );
        let output_path = storage
            .resolver()
            .resolve_bucket_path_for_write(FileBucket::Temp, &relative_path)?;
        let transcode_result = transcoder
            .run_transcode(
                ffmpeg_path,
                &segment.absolute_path,
                &output_path,
                spec,
                probe.has_audio_stream,
            )
            .map_err(|error| {
                let replacements = vec![
                    (
                        ffmpeg_path.to_path_buf(),
                        sidecar_relative_path(FFMPEG_BINARY),
                    ),
                    (segment.absolute_path.clone(), segment.relative_path.clone()),
                    (output_path.clone(), format!("temp/{relative_path}")),
                ];
                format!(
                    "{FFMPEG_TRANSCODE_FAILED}: {}",
                    sanitize_process_message(&error, &replacements)
                )
            });
        if let Err(error) = transcode_result {
            let _ = std::fs::remove_file(&output_path);
            return Err(error);
        }
        if !output_path.is_file() {
            return Err(
                "ffmpeg.transcode_output_missing: normalized segment was not created.".to_string(),
            );
        }
        prepared_paths.push(output_path);
    }
    Ok(prepared_paths)
}

fn cleanup_transcoded_segments(
    paths: &[std::path::PathBuf],
    original_segments: &[ResolvedSegment],
) {
    for path in paths {
        let is_original = original_segments
            .iter()
            .any(|segment| segment.absolute_path == *path);
        if !is_original {
            let _ = std::fs::remove_file(path);
        }
    }
}

fn requires_transcode(probe: &MediaProbeDto, spec: &TranscodeSpec) -> bool {
    !is_mp4_probe(probe)
        || probe.video_codec.as_deref() != Some(spec.video_codec)
        || probe.pixel_format.as_deref() != Some(spec.pixel_format)
        || probe.width != Some(spec.width)
        || probe.height != Some(spec.height)
        || probe
            .fps
            .map(|fps| (fps - f64::from(spec.fps)).abs() <= FPS_EPSILON)
            != Some(true)
        || if spec.include_audio {
            !probe.has_audio_stream
                || probe.audio_codec.as_deref() != Some(spec.audio_codec)
                || probe.sample_rate != Some(spec.sample_rate)
        } else {
            probe.has_audio_stream
        }
}

fn is_mp4_probe(probe: &MediaProbeDto) -> bool {
    let format_name = probe.format_name.as_deref().unwrap_or_default();
    let container = probe.container.as_deref().unwrap_or_default();
    format_name
        .split(',')
        .any(|value| value == "mp4" || value == "mov")
        || matches!(container, "mp4" | "mov")
}

fn build_transcode_args(spec: &TranscodeSpec, input_has_audio: bool) -> Vec<String> {
    let mut args = vec!["-map".to_string(), "0:v:0".to_string()];
    if spec.include_audio {
        args.extend([
            "-map".to_string(),
            if input_has_audio { "0:a:0" } else { "1:a:0" }.to_string(),
        ]);
        if !input_has_audio {
            args.push("-shortest".to_string());
        }
    }
    args.extend([
        "-vf".to_string(),
        format!(
            "scale={}:{}:flags=lanczos,fps={},format={}",
            spec.width, spec.height, spec.fps, spec.pixel_format
        ),
        "-c:v".to_string(),
        spec.encoder.to_string(),
        "-preset".to_string(),
        "veryfast".to_string(),
        "-crf".to_string(),
        "18".to_string(),
    ]);
    if spec.include_audio {
        args.extend([
            "-c:a".to_string(),
            spec.audio_codec.to_string(),
            "-b:a".to_string(),
            spec.audio_bitrate.to_string(),
            "-ar".to_string(),
            spec.sample_rate.to_string(),
        ]);
    } else {
        args.push("-an".to_string());
    }
    args.extend(["-movflags".to_string(), "+faststart".to_string()]);
    args
}

fn build_bgm_mix_args(options: &BgmMixOptions) -> Vec<String> {
    let mut filter = format!("volume={:.3}", options.volume);
    if options.fade_in_seconds > 0.0 {
        filter.push_str(&format!(
            ",afade=t=in:st=0:d={:.3}",
            options.fade_in_seconds
        ));
    }
    if options.fade_out_seconds > 0.0 && options.duration_seconds > options.fade_out_seconds {
        let fade_out_start = (options.duration_seconds - options.fade_out_seconds).max(0.0);
        filter.push_str(&format!(
            ",afade=t=out:st={fade_out_start:.3}:d={:.3}",
            options.fade_out_seconds
        ));
    }

    vec![
        "-filter_complex".to_string(),
        if options.video_has_audio {
            format!("[1:a]{filter}[bgm];[0:a][bgm]amix=inputs=2:duration=first:dropout_transition=0[aout]")
        } else {
            format!("[1:a]{filter}[aout]")
        },
        "-map".to_string(),
        "0:v:0".to_string(),
        "-map".to_string(),
        "[aout]".to_string(),
        "-c:v".to_string(),
        "copy".to_string(),
        "-c:a".to_string(),
        STANDARD_AUDIO_CODEC.to_string(),
        "-b:a".to_string(),
        STANDARD_AUDIO_BITRATE.to_string(),
        "-ar".to_string(),
        STANDARD_SAMPLE_RATE.to_string(),
        "-t".to_string(),
        format!("{:.3}", options.duration_seconds),
        "-movflags".to_string(),
        "+faststart".to_string(),
    ]
}

fn build_concat_filelist(paths: &[std::path::PathBuf]) -> String {
    paths
        .iter()
        .map(|path| format!("file '{}'", escape_concat_path(path)))
        .collect::<Vec<_>>()
        .join("\n")
}

fn escape_concat_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .replace('\'', "'\\''")
}

fn validate_bgm_mix_options(options: &BgmMixOptions) -> Result<(), String> {
    let normalized =
        crate::security::path_guard::PathGuard::validate_relative_path(&options.bgm_relative_path)
            .map_err(|error| format!("{FFMPEG_INVALID_MEDIA}: {error}"))?;
    if !normalized.starts_with("assets/") && !normalized.starts_with("projects/") {
        return Err(format!(
            "{FFMPEG_INVALID_MEDIA}: BGM path must point to a controlled assets or projects bucket."
        ));
    }
    if !options.volume.is_finite() || options.volume < 0.0 || options.volume > 1.0 {
        return Err(format!(
            "{FFMPEG_INVALID_MEDIA}: BGM volume must be between 0 and 1."
        ));
    }
    if !options.fade_in_seconds.is_finite()
        || !options.fade_out_seconds.is_finite()
        || options.fade_in_seconds < 0.0
        || options.fade_out_seconds < 0.0
    {
        return Err(format!(
            "{FFMPEG_INVALID_MEDIA}: BGM fade duration must be zero or greater."
        ));
    }
    if !options.duration_seconds.is_finite() || options.duration_seconds <= 0.0 {
        return Err(format!(
            "{FFMPEG_INVALID_MEDIA}: video duration is required before BGM mixing."
        ));
    }
    Ok(())
}

fn validate_subtitle_path(path: &str) -> Result<(), String> {
    let normalized = crate::security::path_guard::PathGuard::validate_relative_path(path)
        .map_err(|error| format!("{FFMPEG_INVALID_MEDIA}: {error}"))?;
    if !normalized.starts_with("projects/") {
        return Err(format!(
            "{FFMPEG_INVALID_MEDIA}: subtitle path must point to the controlled projects bucket."
        ));
    }
    if !normalized.ends_with("/subtitles/subtitles.json") {
        return Err(format!(
            "{FFMPEG_INVALID_MEDIA}: subtitle path must point to subtitles/subtitles.json."
        ));
    }
    Ok(())
}

fn subtitle_json_to_srt(path: &Path) -> Result<String, String> {
    let content = std::fs::read_to_string(path).map_err(|error| {
        format!(
            "{FFMPEG_INVALID_MEDIA}: subtitle file cannot be read: {}",
            sanitize_process_message(
                &error.to_string(),
                &[(path.to_path_buf(), "subtitles.json".to_string())]
            )
        )
    })?;
    let parsed: Value = serde_json::from_str(&content).map_err(|error| {
        format!(
            "{FFMPEG_INVALID_MEDIA}: subtitle file is not valid JSON: {}",
            truncate_process_text(&error.to_string())
        )
    })?;
    let chunks = parsed
        .get("chunks")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{FFMPEG_INVALID_MEDIA}: subtitle file must contain chunks."))?;
    let mut lines = Vec::new();
    for (index, chunk) in chunks.iter().enumerate() {
        let text = chunk
            .get("text")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("{FFMPEG_INVALID_MEDIA}: subtitle chunk text is required."))?;
        let start = chunk
            .get("startSeconds")
            .or_else(|| chunk.get("start_seconds"))
            .and_then(Value::as_f64)
            .filter(|value| value.is_finite() && *value >= 0.0)
            .ok_or_else(|| format!("{FFMPEG_INVALID_MEDIA}: subtitle chunk start is required."))?;
        let end = chunk
            .get("endSeconds")
            .or_else(|| chunk.get("end_seconds"))
            .and_then(Value::as_f64)
            .filter(|value| value.is_finite() && *value > start)
            .ok_or_else(|| format!("{FFMPEG_INVALID_MEDIA}: subtitle chunk end is required."))?;
        lines.push(format!(
            "{}\n{} --> {}\n{}\n",
            index + 1,
            format_srt_timestamp(start),
            format_srt_timestamp(end),
            text.replace('\r', "").replace('\n', " ")
        ));
    }
    if lines.is_empty() {
        return Err(format!(
            "{FFMPEG_INVALID_MEDIA}: subtitle file must contain at least one chunk."
        ));
    }
    Ok(lines.join("\n"))
}

fn format_srt_timestamp(seconds: f64) -> String {
    let milliseconds = (seconds.max(0.0) * 1000.0).round() as u64;
    let hours = milliseconds / 3_600_000;
    let minutes = (milliseconds % 3_600_000) / 60_000;
    let secs = (milliseconds % 60_000) / 1000;
    let millis = milliseconds % 1000;
    format!("{hours:02}:{minutes:02}:{secs:02},{millis:03}")
}

fn escape_subtitle_filter_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .replace(':', "\\:")
        .replace('\'', "\\'")
        .replace(',', "\\,")
}

fn split_workspace_relative_path(relative_path: &str) -> Result<(FileBucket, &str), String> {
    let (bucket_segment, bucket_relative_path) = relative_path
        .split_once('/')
        .ok_or_else(|| format!("{FFMPEG_INVALID_MEDIA}: media path must include a bucket."))?;
    if bucket_relative_path.trim().is_empty() {
        return Err(format!(
            "{FFMPEG_INVALID_MEDIA}: media path cannot be empty."
        ));
    }

    let bucket = match bucket_segment {
        "projects" => FileBucket::Project,
        "assets" => FileBucket::Asset,
        "outputs" => FileBucket::Output,
        "cache" => FileBucket::Cache,
        "temp" => FileBucket::Temp,
        _ => {
            return Err(format!(
                "{FFMPEG_INVALID_MEDIA}: media path must point to a controlled workspace bucket."
            ))
        }
    };

    Ok((bucket, bucket_relative_path))
}

fn parse_ffprobe_json(
    relative_path: &str,
    media_kind: &str,
    raw_json: &str,
) -> Result<MediaProbeDto, String> {
    let output: FfprobeOutput = serde_json::from_str(raw_json).map_err(|error| {
        format!(
            "{FFMPEG_PROBE_FAILED}: ffprobe returned invalid JSON: {}",
            truncate_process_text(&error.to_string())
        )
    })?;
    let streams = output.streams.unwrap_or_default();
    let video_stream = streams.iter().find(|stream| stream.codec_type == "video");
    let audio_stream = streams.iter().find(|stream| stream.codec_type == "audio");
    let format = output.format.unwrap_or_default();
    let duration_seconds = parse_duration(format.duration.as_deref())
        .or_else(|| video_stream.and_then(|stream| parse_duration(stream.duration.as_deref())))
        .or_else(|| audio_stream.and_then(|stream| parse_duration(stream.duration.as_deref())))
        .unwrap_or(0.0);

    if media_kind == "video" && video_stream.is_none() {
        return Err(format!(
            "{FFMPEG_INVALID_MEDIA}: video stream is missing for {relative_path}."
        ));
    }
    if duration_seconds <= 0.0 {
        return Err(format!(
            "{FFMPEG_INVALID_MEDIA}: media duration is missing or invalid for {relative_path}."
        ));
    }

    let width = video_stream.and_then(|stream| stream.width);
    let height = video_stream.and_then(|stream| stream.height);
    let fps = video_stream
        .and_then(|stream| parse_fps(stream.avg_frame_rate.as_deref()))
        .or_else(|| video_stream.and_then(|stream| parse_fps(stream.r_frame_rate.as_deref())));
    let sample_rate = audio_stream.and_then(|stream| parse_u32(stream.sample_rate.as_deref()));
    let channels = audio_stream.and_then(|stream| stream.channels);
    let bit_rate = parse_u64(format.bit_rate.as_deref())
        .or_else(|| video_stream.and_then(|stream| parse_u64(stream.bit_rate.as_deref())))
        .or_else(|| audio_stream.and_then(|stream| parse_u64(stream.bit_rate.as_deref())));
    let format_name = format.format_name.filter(|value| !value.trim().is_empty());
    let container = format_name
        .as_deref()
        .and_then(|value| value.split(',').next())
        .filter(|value| !value.trim().is_empty())
        .map(ToString::to_string);

    Ok(MediaProbeDto {
        path: relative_path.to_string(),
        media_kind: media_kind.to_string(),
        container,
        format_name,
        duration_seconds,
        width,
        height,
        fps,
        video_codec: video_stream.and_then(|stream| stream.codec_name.clone()),
        pixel_format: video_stream.and_then(|stream| stream.pix_fmt.clone()),
        audio_codec: audio_stream.and_then(|stream| stream.codec_name.clone()),
        sample_rate,
        channels,
        bit_rate,
        has_video_stream: video_stream.is_some(),
        has_audio_stream: audio_stream.is_some(),
    })
}

fn summarize_unready_sidecars(status: &FfmpegSidecarStatusDto) -> String {
    let mut missing = Vec::new();
    if !status.ffmpeg.executable {
        missing.push(status.ffmpeg.relative_path.as_str());
    }
    if !status.ffprobe.executable {
        missing.push(status.ffprobe.relative_path.as_str());
    }

    if missing.is_empty() {
        "FFmpeg sidecar is not ready.".to_string()
    } else {
        format!("FFmpeg sidecar is not ready: {}", missing.join(", "))
    }
}

fn sidecar_relative_path(binary_name: &str) -> String {
    format!("sidecars/{binary_name}")
}

fn sanitize_sidecar_message(message: &str, absolute_path: Option<&Path>) -> String {
    let replacements = absolute_path
        .map(|path| {
            let relative = path
                .file_name()
                .and_then(|file_name| file_name.to_str())
                .map(sidecar_relative_path)
                .unwrap_or_else(|| "sidecars/<binary>".to_string());
            vec![(path.to_path_buf(), relative)]
        })
        .unwrap_or_default();
    sanitize_process_message(message, &replacements)
}

fn sanitize_probe_message(message: &str, absolute_path: Option<&Path>) -> String {
    let replacements = absolute_path
        .and_then(|path| {
            path.file_name()
                .and_then(|file_name| file_name.to_str())
                .map(|file_name| vec![(path.to_path_buf(), file_name.to_string())])
        })
        .unwrap_or_default();
    sanitize_process_message(message, &replacements)
}

fn sanitize_process_message(message: &str, replacements: &[(PathBuf, String)]) -> String {
    let mut sanitized = redact_text(message);
    for (path, replacement) in replacements {
        sanitized = replace_path_with_replacement(sanitized, path, replacement);
    }
    truncate_process_text(&sanitized)
}

fn replace_path_with_replacement(mut value: String, path: &Path, replacement: &str) -> String {
    let display_path = path.display().to_string();
    value = value.replace(&display_path, replacement);
    if let Some(path_text) = path.to_str() {
        value = value.replace(path_text, replacement);
        value = value.replace(&path_text.replace('\\', "/"), replacement);
        value = value.replace(&path_text.replace('/', "\\"), replacement);
    }
    value
}

fn parse_duration(value: Option<&str>) -> Option<f64> {
    value
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|value| value.is_finite() && *value > 0.0)
}

fn parse_fps(value: Option<&str>) -> Option<f64> {
    let value = value?;
    if value == "0/0" {
        return None;
    }
    if let Some((numerator, denominator)) = value.split_once('/') {
        let numerator = numerator.parse::<f64>().ok()?;
        let denominator = denominator.parse::<f64>().ok()?;
        if denominator == 0.0 {
            return None;
        }
        let fps = numerator / denominator;
        return fps.is_finite().then_some(fps).filter(|value| *value > 0.0);
    }

    value
        .parse::<f64>()
        .ok()
        .filter(|fps| fps.is_finite() && *fps > 0.0)
}

fn parse_u32(value: Option<&str>) -> Option<u32> {
    value.and_then(|value| value.parse::<u32>().ok())
}

fn parse_u64(value: Option<&str>) -> Option<u64> {
    value.and_then(|value| value.parse::<u64>().ok())
}

fn sanitize_file_segment(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    let trimmed = sanitized.trim_matches('.').trim_matches('_');
    if trimmed.is_empty() {
        "item".to_string()
    } else {
        trimmed.to_string()
    }
}

fn first_non_empty_line(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(ToString::to_string)
}

fn truncate_process_text(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let lines = trimmed.lines().collect::<Vec<_>>();
    let line_start = lines.len().saturating_sub(PROCESS_TEXT_MAX_LINES);
    let line_limited = lines[line_start..].join("\n");
    if line_limited.len() <= PROCESS_TEXT_MAX_BYTES {
        return line_limited;
    }

    take_last_utf8_bytes(&line_limited, PROCESS_TEXT_MAX_BYTES)
}

fn take_last_utf8_bytes(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_string();
    }

    let mut start = value.len() - max_bytes;
    while start < value.len() && !value.is_char_boundary(start) {
        start += 1;
    }
    value[start..].to_string()
}

fn current_timestamp_string() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_secs();
    seconds.to_string()
}

#[derive(Debug, Deserialize, Default)]
struct FfprobeOutput {
    streams: Option<Vec<FfprobeStream>>,
    format: Option<FfprobeFormat>,
}

#[derive(Debug, Deserialize, Default)]
struct FfprobeFormat {
    format_name: Option<String>,
    duration: Option<String>,
    bit_rate: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FfprobeStream {
    codec_type: String,
    codec_name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    avg_frame_rate: Option<String>,
    r_frame_rate: Option<String>,
    duration: Option<String>,
    sample_rate: Option<String>,
    channels: Option<u32>,
    bit_rate: Option<String>,
    pix_fmt: Option<String>,
    #[serde(flatten)]
    _extra: Value,
}

#[cfg(test)]
mod tests {
    use super::{
        build_concat_filelist, check_ffmpeg_sidecars_with_runner, concat_segments_with_runner,
        mix_bgm_into_video_with_runner, parse_ffprobe_json, probe_media_with_runner,
        require_ffmpeg_sidecars, sanitize_process_message, BgmMixOptions, FFMPEG_BINARY,
        FFPROBE_BINARY, PROCESS_TEXT_MAX_BYTES, PROCESS_TEXT_MAX_LINES,
    };
    use std::collections::HashMap;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct FakeRunner {
        versions_by_name: HashMap<String, Result<String, String>>,
    }

    struct FakeProbeRunner {
        output: Result<String, String>,
    }

    #[derive(Default)]
    struct FakeConcatRunner {
        should_fail: bool,
        filelist_content: Arc<Mutex<Option<String>>>,
    }

    #[derive(Default)]
    struct FakeTranscodeRunner {
        should_fail: bool,
        calls: Arc<Mutex<Vec<FakeTranscodeCall>>>,
    }

    #[derive(Default)]
    struct FakeBgmMixRunner {
        should_fail: bool,
        calls: Arc<Mutex<Vec<FakeBgmMixCall>>>,
    }

    #[derive(Default)]
    struct FakeSubtitleBurnRunner {
        should_fail: bool,
        calls: Arc<Mutex<Vec<FakeSubtitleBurnCall>>>,
    }

    #[derive(Debug, Clone)]
    struct FakeTranscodeCall {
        input_path: PathBuf,
        output_path: PathBuf,
        args: Vec<String>,
        input_has_audio: bool,
    }

    #[derive(Debug, Clone)]
    struct FakeBgmMixCall {
        video_path: PathBuf,
        bgm_path: PathBuf,
        output_path: PathBuf,
        args: Vec<String>,
    }

    #[derive(Debug, Clone)]
    struct FakeSubtitleBurnCall {
        video_path: PathBuf,
        subtitle_path: PathBuf,
        output_path: PathBuf,
        subtitle_content: String,
    }

    impl super::VersionRunner for FakeRunner {
        fn run_version(&self, binary_path: &Path) -> Result<String, String> {
            let name = binary_path
                .file_name()
                .and_then(|value| value.to_str())
                .expect("binary path should have a file name");
            self.versions_by_name
                .get(name)
                .cloned()
                .unwrap_or_else(|| Err("unexpected binary".to_string()))
        }
    }

    impl super::ProbeRunner for FakeProbeRunner {
        fn run_probe(&self, _ffprobe_path: &Path, _media_path: &Path) -> Result<String, String> {
            self.output.clone()
        }
    }

    impl super::ConcatRunner for FakeConcatRunner {
        fn run_concat(
            &self,
            _ffmpeg_path: &Path,
            filelist_path: &Path,
            output_path: &Path,
        ) -> Result<(), String> {
            if self.should_fail {
                return Err(format!("failed writing {}", output_path.display()));
            }
            let filelist_content =
                fs::read_to_string(filelist_path).map_err(|error| error.to_string())?;
            *self
                .filelist_content
                .lock()
                .expect("filelist record should lock") = Some(filelist_content);
            fs::write(output_path, "final").map_err(|error| error.to_string())
        }
    }

    impl super::TranscodeRunner for FakeTranscodeRunner {
        fn run_transcode(
            &self,
            _ffmpeg_path: &Path,
            input_path: &Path,
            output_path: &Path,
            spec: &super::TranscodeSpec,
            input_has_audio: bool,
        ) -> Result<(), String> {
            self.calls
                .lock()
                .expect("transcode calls should lock")
                .push(FakeTranscodeCall {
                    input_path: input_path.to_path_buf(),
                    output_path: output_path.to_path_buf(),
                    args: super::build_transcode_args(spec, input_has_audio),
                    input_has_audio,
                });
            if self.should_fail {
                return Err(format!("failed transcoding {}", output_path.display()));
            }
            fs::write(output_path, "normalized").map_err(|error| error.to_string())
        }
    }

    impl super::BgmMixRunner for FakeBgmMixRunner {
        fn run_mix_bgm(
            &self,
            _ffmpeg_path: &Path,
            video_path: &Path,
            bgm_path: &Path,
            output_path: &Path,
            options: &super::BgmMixOptions,
        ) -> Result<(), String> {
            self.calls
                .lock()
                .expect("bgm mix calls should lock")
                .push(FakeBgmMixCall {
                    video_path: video_path.to_path_buf(),
                    bgm_path: bgm_path.to_path_buf(),
                    output_path: output_path.to_path_buf(),
                    args: super::build_bgm_mix_args(options),
                });
            if self.should_fail {
                return Err(format!("failed mixing {}", output_path.display()));
            }
            fs::write(output_path, "mixed").map_err(|error| error.to_string())
        }
    }

    impl super::SubtitleBurnRunner for FakeSubtitleBurnRunner {
        fn run_burn_subtitles(
            &self,
            _ffmpeg_path: &Path,
            video_path: &Path,
            subtitle_path: &Path,
            output_path: &Path,
        ) -> Result<(), String> {
            self.calls
                .lock()
                .expect("subtitle burn calls should lock")
                .push(FakeSubtitleBurnCall {
                    video_path: video_path.to_path_buf(),
                    subtitle_path: subtitle_path.to_path_buf(),
                    output_path: output_path.to_path_buf(),
                    subtitle_content: fs::read_to_string(subtitle_path)
                        .expect("temporary srt should be readable"),
                });
            if self.should_fail {
                return Err(format!("failed burning {}", output_path.display()));
            }
            fs::write(output_path, "subtitle").map_err(|error| error.to_string())
        }
    }

    #[test]
    fn missing_sidecars_return_not_ready_without_absolute_paths() {
        let root = test_root("missing");

        let status = check_ffmpeg_sidecars_with_runner(&root, &FakeRunner::default())
            .expect("missing sidecars should produce a status");

        assert!(!status.ready);
        assert_eq!(status.ffmpeg.relative_path, "sidecars/ffmpeg.exe");
        assert!(!status.ffmpeg.exists);
        assert_eq!(
            status.ffmpeg.error_code.as_deref(),
            Some("ffmpeg.not_found")
        );
        assert_no_absolute_path(&status.ffmpeg.message, &root);
        assert_eq!(
            status.ffprobe.error_code.as_deref(),
            Some("ffmpeg.not_found")
        );
        assert_no_absolute_path(&status.ffprobe.message, &root);

        cleanup(root);
    }

    #[test]
    fn fake_sidecars_can_report_versions() {
        let root = test_root("ready");
        let sidecars = root.join("sidecars");
        fs::create_dir_all(&sidecars).expect("sidecars dir should exist");
        fs::write(sidecars.join(FFMPEG_BINARY), "fake").expect("ffmpeg fake should write");
        fs::write(sidecars.join(FFPROBE_BINARY), "fake").expect("ffprobe fake should write");

        let status = check_ffmpeg_sidecars_with_runner(
            &root,
            &FakeRunner::with_versions([
                (FFMPEG_BINARY, Ok("ffmpeg version 6.1 fake".to_string())),
                (FFPROBE_BINARY, Ok("ffprobe version 6.1 fake".to_string())),
            ]),
        )
        .expect("fake sidecars should be ready");

        assert!(status.ready);
        assert!(status.ffmpeg.executable);
        assert!(status.ffprobe.executable);
        assert_eq!(
            status.ffmpeg.version.as_deref(),
            Some("ffmpeg version 6.1 fake")
        );
        assert_eq!(
            status.ffprobe.version.as_deref(),
            Some("ffprobe version 6.1 fake")
        );
        assert!(status.ffmpeg.message.is_none());

        cleanup(root);
    }

    #[test]
    fn require_sidecars_blocks_composition_when_missing() {
        let root = test_root("require_missing");

        let error = require_ffmpeg_sidecars(&root).expect_err("missing sidecars should block");

        assert!(error.starts_with("ffmpeg.not_found:"));
        assert!(error.contains("sidecars/ffmpeg.exe"));
        assert!(error.contains("sidecars/ffprobe.exe"));
        assert!(!error.contains(&root.display().to_string()));

        cleanup(root);
    }

    #[test]
    fn parses_ffprobe_video_json() {
        let probe = parse_ffprobe_json(
            "projects/project_a/videos/seg.mp4",
            "video",
            r#"
            {
              "streams": [
                {
                  "codec_type": "video",
                  "codec_name": "h264",
                  "pix_fmt": "yuv420p",
                  "width": 1280,
                  "height": 720,
                  "avg_frame_rate": "30000/1001",
                  "bit_rate": "1200000"
                },
                {
                  "codec_type": "audio",
                  "codec_name": "aac",
                  "sample_rate": "44100",
                  "channels": 2,
                  "bit_rate": "128000"
                }
              ],
              "format": {
                "format_name": "mov,mp4,m4a,3gp,3g2,mj2",
                "duration": "4.250000",
                "bit_rate": "1400000"
              }
            }
            "#,
        )
        .expect("probe json should parse");

        assert_eq!(probe.path, "projects/project_a/videos/seg.mp4");
        assert_eq!(probe.media_kind, "video");
        assert_eq!(probe.container.as_deref(), Some("mov"));
        assert_eq!(probe.video_codec.as_deref(), Some("h264"));
        assert_eq!(probe.pixel_format.as_deref(), Some("yuv420p"));
        assert_eq!(probe.audio_codec.as_deref(), Some("aac"));
        assert_eq!(probe.width, Some(1280));
        assert_eq!(probe.height, Some(720));
        assert_eq!(probe.sample_rate, Some(44100));
        assert_eq!(probe.channels, Some(2));
        assert_eq!(probe.bit_rate, Some(1400000));
        assert!(probe.has_video_stream);
        assert!(probe.has_audio_stream);
        assert!((probe.duration_seconds - 4.25).abs() < 0.001);
        assert!((probe.fps.unwrap() - 29.970).abs() < 0.01);
    }

    #[test]
    fn rejects_video_probe_without_video_stream() {
        let error = parse_ffprobe_json(
            "projects/project_a/videos/audio.mp4",
            "video",
            r#"
            {
              "streams": [
                { "codec_type": "audio", "codec_name": "aac", "sample_rate": "44100" }
              ],
              "format": { "format_name": "mp4", "duration": "4.0" }
            }
            "#,
        )
        .expect_err("video media without video stream should be rejected");

        assert!(error.starts_with("ffmpeg.invalid_media:"));
    }

    #[test]
    fn probe_media_rejects_escaped_relative_path() {
        let root = test_root("probe_escaped");
        let sidecars = root.join("sidecars");
        fs::create_dir_all(&sidecars).expect("sidecars dir should exist");
        fs::write(sidecars.join(FFMPEG_BINARY), "fake").expect("ffmpeg fake should write");
        fs::write(sidecars.join(FFPROBE_BINARY), "fake").expect("ffprobe fake should write");

        let error = probe_media_with_runner(
            &root,
            "projects/../outside.mp4",
            Some("video"),
            &FakeProbeRunner {
                output: Ok("{}".to_string()),
            },
        )
        .expect_err("escaped path should be rejected");

        assert!(error.starts_with("ffmpeg.invalid_media:"));
        assert!(!error.contains(&root.display().to_string()));

        cleanup(root);
    }

    #[test]
    fn probe_media_uses_ffprobe_sidecar_and_controlled_path() {
        let root = test_root("probe_ready");
        let sidecars = root.join("sidecars");
        let videos = root.join("projects/project_a/videos");
        fs::create_dir_all(&sidecars).expect("sidecars dir should exist");
        fs::create_dir_all(&videos).expect("videos dir should exist");
        fs::write(sidecars.join(FFMPEG_BINARY), "fake").expect("ffmpeg fake should write");
        fs::write(sidecars.join(FFPROBE_BINARY), "fake").expect("ffprobe fake should write");
        fs::write(videos.join("seg.mp4"), "fake").expect("video fake should write");

        let probe = probe_media_with_runner(
            &root,
            "projects/project_a/videos/seg.mp4",
            Some("video"),
            &FakeProbeRunner {
                output: Ok(r#"{ "streams": [{ "codec_type": "video", "codec_name": "h264", "width": 720, "height": 1280, "avg_frame_rate": "30/1" }], "format": { "format_name": "mp4", "duration": "3.0" } }"#.to_string()),
            },
        )
        .expect("controlled media should probe");

        assert_eq!(probe.path, "projects/project_a/videos/seg.mp4");
        assert_eq!(probe.duration_seconds, 3.0);
        assert_eq!(probe.fps, Some(30.0));

        cleanup(root);
    }

    #[test]
    fn concat_filelist_escapes_paths_for_demuxer() {
        let filelist = build_concat_filelist(&[
            PathBuf::from("D:/workspace/projects/a/video one.mp4"),
            PathBuf::from("D:/workspace/projects/a/quote's.mp4"),
        ]);

        assert!(filelist.contains("file 'D:/workspace/projects/a/video one.mp4'"));
        assert!(filelist.contains("quote'\\''s.mp4"));
    }

    #[test]
    fn concat_segments_writes_output_relative_path_and_removes_filelist() {
        let root = test_root("concat_ready");
        let sidecars = root.join("sidecars");
        let videos = root.join("projects/project_a/videos");
        fs::create_dir_all(&sidecars).expect("sidecars dir should exist");
        fs::create_dir_all(&videos).expect("videos dir should exist");
        fs::write(sidecars.join(FFMPEG_BINARY), "fake").expect("ffmpeg fake should write");
        fs::write(sidecars.join(FFPROBE_BINARY), "fake").expect("ffprobe fake should write");
        fs::write(videos.join("seg_1.mp4"), "segment").expect("segment should write");

        let output = concat_segments_with_runner(
            &root,
            "project_a",
            "composition_1",
            &["projects/project_a/videos/seg_1.mp4".to_string()],
            &FakeConcatRunner::default(),
            &FakeTranscodeRunner::default(),
            None,
        )
        .expect("concat should succeed");

        assert_eq!(output, "outputs/exports/project_a/composition_1_final.mp4");
        assert!(root.join(&output).is_file());
        assert!(!root
            .join("temp/composition/project_a/composition_1_concat.txt")
            .exists());

        cleanup(root);
    }

    #[test]
    fn concat_segments_maps_runner_failure_without_absolute_output_path() {
        let root = test_root("concat_fail");
        let sidecars = root.join("sidecars");
        let videos = root.join("projects/project_a/videos");
        fs::create_dir_all(&sidecars).expect("sidecars dir should exist");
        fs::create_dir_all(&videos).expect("videos dir should exist");
        fs::write(sidecars.join(FFMPEG_BINARY), "fake").expect("ffmpeg fake should write");
        fs::write(sidecars.join(FFPROBE_BINARY), "fake").expect("ffprobe fake should write");
        fs::write(videos.join("seg_1.mp4"), "segment").expect("segment should write");

        let error = concat_segments_with_runner(
            &root,
            "project_a",
            "composition_1",
            &["projects/project_a/videos/seg_1.mp4".to_string()],
            &FakeConcatRunner {
                should_fail: true,
                ..Default::default()
            },
            &FakeTranscodeRunner::default(),
            None,
        )
        .expect_err("concat failure should be mapped");

        assert!(error.starts_with("ffmpeg.concat_failed:"));
        assert!(!error.contains(&root.display().to_string()));

        cleanup(root);
    }

    #[test]
    fn standard_probe_does_not_transcode_before_concat() {
        let root = test_root("concat_no_transcode");
        write_concat_fixture(&root, &["seg_1.mp4"]);
        let concat_runner = FakeConcatRunner::default();
        let transcode_runner = FakeTranscodeRunner::default();

        let output = concat_segments_with_runner(
            &root,
            "project_a",
            "composition_1",
            &["projects/project_a/videos/seg_1.mp4".to_string()],
            &concat_runner,
            &transcode_runner,
            Some(&[standard_probe("projects/project_a/videos/seg_1.mp4")]),
        )
        .expect("standard segment should concat without transcode");

        assert_eq!(output, "outputs/exports/project_a/composition_1_final.mp4");
        assert!(transcode_runner
            .calls
            .lock()
            .expect("calls should lock")
            .is_empty());
        let filelist = concat_runner
            .filelist_content
            .lock()
            .expect("filelist should lock")
            .clone()
            .expect("filelist should be captured");
        assert!(filelist.contains("projects/project_a/videos/seg_1.mp4"));

        cleanup(root);
    }

    #[test]
    fn non_standard_probe_transcodes_to_temp_before_concat_and_cleans_temp_files() {
        let root = test_root("concat_transcode");
        write_concat_fixture(&root, &["seg_1.mp4", "seg_2.mp4"]);
        let concat_runner = FakeConcatRunner::default();
        let transcode_runner = FakeTranscodeRunner::default();
        let mut non_standard = standard_probe("projects/project_a/videos/seg_1.mp4");
        non_standard.video_codec = Some("hevc".to_string());
        non_standard.pixel_format = Some("yuv444p".to_string());
        non_standard.fps = Some(24.0);
        let probes = vec![
            non_standard,
            standard_probe("projects/project_a/videos/seg_2.mp4"),
        ];

        let output = concat_segments_with_runner(
            &root,
            "project_a",
            "composition_1",
            &[
                "projects/project_a/videos/seg_1.mp4".to_string(),
                "projects/project_a/videos/seg_2.mp4".to_string(),
            ],
            &concat_runner,
            &transcode_runner,
            Some(&probes),
        )
        .expect("non-standard segment should transcode then concat");

        assert_eq!(output, "outputs/exports/project_a/composition_1_final.mp4");
        let calls = transcode_runner
            .calls
            .lock()
            .expect("calls should lock")
            .clone();
        assert_eq!(calls.len(), 1);
        assert!(calls[0]
            .output_path
            .ends_with("temp/composition/project_a/composition_1/segment_0001_normalized.mp4"));
        assert!(calls[0]
            .args
            .windows(2)
            .any(|pair| pair == ["-c:v", "libx264"]));
        assert!(calls[0].args.windows(2).any(|pair| pair == ["-crf", "18"]));
        assert!(calls[0]
            .args
            .windows(2)
            .any(|pair| pair == ["-ar", "44100"]));
        assert!(calls[0].input_has_audio);
        assert!(calls[0]
            .input_path
            .ends_with("projects/project_a/videos/seg_1.mp4"));
        let filelist = concat_runner
            .filelist_content
            .lock()
            .expect("filelist should lock")
            .clone()
            .expect("filelist should be captured");
        assert!(filelist.contains("segment_0001_normalized.mp4"));
        assert!(filelist.contains("projects/project_a/videos/seg_2.mp4"));
        assert!(!root
            .join("temp/composition/project_a/composition_1/segment_0001_normalized.mp4")
            .exists());

        cleanup(root);
    }

    #[test]
    fn transcode_failure_is_mapped_without_absolute_path_and_removes_partial_output() {
        let root = test_root("concat_transcode_fail");
        write_concat_fixture(&root, &["seg_1.mp4"]);
        let mut probe = standard_probe("projects/project_a/videos/seg_1.mp4");
        probe.video_codec = Some("hevc".to_string());
        probe.fps = Some(24.0);
        let transcode_runner = FakeTranscodeRunner {
            should_fail: true,
            ..Default::default()
        };

        let error = concat_segments_with_runner(
            &root,
            "project_a",
            "composition_1",
            &["projects/project_a/videos/seg_1.mp4".to_string()],
            &FakeConcatRunner::default(),
            &transcode_runner,
            Some(&[probe]),
        )
        .expect_err("transcode failure should be mapped");

        assert!(error.starts_with("ffmpeg.transcode_failed:"));
        assert!(!error.contains(&root.display().to_string()));
        assert!(!root
            .join("temp/composition/project_a/composition_1/segment_0001_normalized.mp4")
            .exists());

        cleanup(root);
    }

    #[test]
    fn process_log_sanitizer_redacts_secrets_paths_and_truncates_tail() {
        let root = test_root("sanitize_process_log");
        let input_path = root.join("projects/project_a/videos/private_seg.mp4");
        let output_path = root.join("outputs/exports/project_a/final.mp4");
        let mut stderr = (0..260)
            .map(|index| format!("line {index:03} {}", root.display()))
            .collect::<Vec<_>>()
            .join("\n");
        stderr.push_str(&format!(
            "\nAuthorization: Bearer sk-live-secret-token-123456\nfailed input {} output {}",
            input_path.display(),
            output_path.display()
        ));

        let sanitized = sanitize_process_message(
            &stderr,
            &[
                (root.clone(), "<workspace>".to_string()),
                (
                    input_path,
                    "projects/project_a/videos/private_seg.mp4".to_string(),
                ),
                (
                    output_path,
                    "outputs/exports/project_a/final.mp4".to_string(),
                ),
            ],
        );

        assert!(!sanitized.contains("sk-live"));
        assert!(sanitized.contains("***REDACTED***"));
        assert!(!sanitized.contains(&root.display().to_string()));
        assert!(sanitized.contains("<workspace>"));
        assert!(sanitized.contains("projects/project_a/videos/private_seg.mp4"));
        assert!(sanitized.lines().count() <= PROCESS_TEXT_MAX_LINES);
        assert!(sanitized.len() <= PROCESS_TEXT_MAX_BYTES);

        cleanup(root);
    }

    #[test]
    fn process_log_sanitizer_keeps_last_lines_when_too_many_lines() {
        let stderr = (0..250)
            .map(|index| format!("line {index:03}"))
            .collect::<Vec<_>>()
            .join("\n");

        let sanitized = sanitize_process_message(&stderr, &[]);

        assert!(!sanitized.contains("line 000"));
        assert!(sanitized.contains("line 050"));
        assert!(sanitized.contains("line 249"));
        assert_eq!(sanitized.lines().count(), PROCESS_TEXT_MAX_LINES);
    }

    #[test]
    fn bgm_mix_writes_output_relative_path_and_builds_safe_filter() {
        let root = test_root("bgm_mix");
        write_concat_fixture(&root, &["seg_1.mp4"]);
        let bgm_dir = root.join("assets/bgm");
        let final_dir = root.join("outputs/exports/project_a");
        fs::create_dir_all(&bgm_dir).expect("bgm dir should exist");
        fs::create_dir_all(&final_dir).expect("final dir should exist");
        fs::write(bgm_dir.join("music.mp3"), "bgm").expect("bgm should write");
        fs::write(final_dir.join("composition_1_final.mp4"), "final").expect("final should write");
        let runner = FakeBgmMixRunner::default();

        let output = mix_bgm_into_video_with_runner(
            &root,
            "project_a",
            "composition_1",
            "outputs/exports/project_a/composition_1_final.mp4",
            &BgmMixOptions {
                bgm_relative_path: "assets/bgm/music.mp3".to_string(),
                volume: 0.18,
                loop_bgm: true,
                fade_in_seconds: 1.5,
                fade_out_seconds: 2.0,
                duration_seconds: 8.0,
                video_has_audio: true,
            },
            &runner,
        )
        .expect("bgm should mix");

        assert_eq!(
            output,
            "outputs/exports/project_a/composition_1_final_bgm.mp4"
        );
        let calls = runner.calls.lock().expect("calls should lock").clone();
        assert_eq!(calls.len(), 1);
        assert!(calls[0]
            .video_path
            .ends_with("outputs/exports/project_a/composition_1_final.mp4"));
        assert!(calls[0].bgm_path.ends_with("assets/bgm/music.mp3"));
        assert!(calls[0]
            .output_path
            .ends_with("outputs/exports/project_a/composition_1_final_bgm.mp4"));
        let args = calls[0].args.join(" ");
        assert!(args.contains("volume=0.180"));
        assert!(args.contains("afade=t=in:st=0:d=1.500"));
        assert!(args.contains("afade=t=out:st=6.000:d=2.000"));
        assert!(args.contains("amix=inputs=2:duration=first"));
        assert!(args.contains("-t 8.000"));

        cleanup(root);
    }

    #[test]
    fn bgm_mix_rejects_uncontrolled_bgm_path() {
        let error = mix_bgm_into_video_with_runner(
            Path::new("D:/workspace"),
            "project_a",
            "composition_1",
            "outputs/exports/project_a/composition_1_final.mp4",
            &BgmMixOptions {
                bgm_relative_path: "../music.mp3".to_string(),
                volume: 0.18,
                loop_bgm: true,
                fade_in_seconds: 0.0,
                fade_out_seconds: 0.0,
                duration_seconds: 8.0,
                video_has_audio: false,
            },
            &FakeBgmMixRunner::default(),
        )
        .expect_err("escaped bgm path should fail");

        assert!(error.starts_with("ffmpeg.invalid_media:"));
    }

    #[test]
    fn subtitle_burn_converts_subtitle_json_to_temp_srt() {
        let root = test_root("subtitle_burn");
        write_concat_fixture(&root, &["seg_1.mp4"]);
        let final_dir = root.join("outputs/exports/project_a");
        let subtitle_dir = root.join("projects/project_a/subtitles");
        fs::create_dir_all(&final_dir).expect("final dir should exist");
        fs::create_dir_all(&subtitle_dir).expect("subtitle dir should exist");
        fs::write(final_dir.join("composition_1_final.mp4"), "final").expect("final should write");
        fs::write(
            subtitle_dir.join("subtitles.json"),
            r#"{
              "chunks": [
                { "text": "第一句字幕", "startSeconds": 0.0, "endSeconds": 1.5 },
                { "text": "第二句字幕", "startSeconds": 1.5, "endSeconds": 3.0 }
              ]
            }"#,
        )
        .expect("subtitle json should write");
        let runner = FakeSubtitleBurnRunner::default();

        let output = super::burn_subtitles_into_video_with_runner(
            &root,
            "project_a",
            "composition_1",
            "outputs/exports/project_a/composition_1_final.mp4",
            "projects/project_a/subtitles/subtitles.json",
            &runner,
        )
        .expect("subtitle burn should pass");

        assert_eq!(
            output,
            "outputs/exports/project_a/composition_1_final_subtitle.mp4"
        );
        let calls = runner.calls.lock().expect("calls should lock").clone();
        assert_eq!(calls.len(), 1);
        assert!(calls[0]
            .video_path
            .ends_with("outputs/exports/project_a/composition_1_final.mp4"));
        assert!(calls[0]
            .subtitle_path
            .ends_with("temp/composition/project_a/composition_1_subtitles.srt"));
        assert!(calls[0]
            .output_path
            .ends_with("outputs/exports/project_a/composition_1_final_subtitle.mp4"));
        assert!(calls[0]
            .subtitle_content
            .contains("00:00:00,000 --> 00:00:01,500"));
        assert!(calls[0].subtitle_content.contains("第一句字幕"));
        assert!(!root
            .join("temp/composition/project_a/composition_1_subtitles.srt")
            .exists());

        cleanup(root);
    }

    #[test]
    fn subtitle_burn_rejects_uncontrolled_subtitle_path() {
        let error = super::burn_subtitles_into_video_with_runner(
            Path::new("D:/workspace"),
            "project_a",
            "composition_1",
            "outputs/exports/project_a/composition_1_final.mp4",
            "assets/subtitles/subtitles.json",
            &FakeSubtitleBurnRunner::default(),
        )
        .expect_err("subtitle path outside projects should fail");

        assert!(error.starts_with("ffmpeg.invalid_media:"));
    }

    impl FakeRunner {
        fn with_versions<const N: usize>(
            entries: [(&'static str, Result<String, String>); N],
        ) -> Self {
            Self {
                versions_by_name: entries
                    .into_iter()
                    .map(|(name, result)| (name.to_string(), result))
                    .collect(),
            }
        }
    }

    impl Default for FakeRunner {
        fn default() -> Self {
            Self {
                versions_by_name: HashMap::new(),
            }
        }
    }

    fn write_concat_fixture(root: &Path, segment_names: &[&str]) {
        let sidecars = root.join("sidecars");
        let videos = root.join("projects/project_a/videos");
        fs::create_dir_all(&sidecars).expect("sidecars dir should exist");
        fs::create_dir_all(&videos).expect("videos dir should exist");
        fs::write(sidecars.join(FFMPEG_BINARY), "fake").expect("ffmpeg fake should write");
        fs::write(sidecars.join(FFPROBE_BINARY), "fake").expect("ffprobe fake should write");
        for segment_name in segment_names {
            fs::write(videos.join(segment_name), "segment").expect("segment should write");
        }
    }

    fn standard_probe(path: &str) -> crate::domain::media::MediaProbeDto {
        crate::domain::media::MediaProbeDto {
            path: path.to_string(),
            media_kind: "video".to_string(),
            container: Some("mov".to_string()),
            format_name: Some("mov,mp4,m4a,3gp,3g2,mj2".to_string()),
            duration_seconds: 4.0,
            width: Some(1280),
            height: Some(720),
            fps: Some(30.0),
            video_codec: Some("h264".to_string()),
            pixel_format: Some("yuv420p".to_string()),
            audio_codec: Some("aac".to_string()),
            sample_rate: Some(44_100),
            channels: Some(2),
            bit_rate: Some(1_400_000),
            has_video_stream: true,
            has_audio_stream: true,
        }
    }

    fn assert_no_absolute_path(message: &Option<String>, root: &Path) {
        let Some(message) = message else {
            return;
        };
        assert!(!message.contains(&root.display().to_string()));
    }

    fn test_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "vt-ai-short-video-maker-ffmpeg-{name}-{}-{nanos}",
            std::process::id()
        ))
    }

    fn cleanup(path: PathBuf) {
        let _ = fs::remove_dir_all(path);
    }
}
