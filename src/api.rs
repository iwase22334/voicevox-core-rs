#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!("./voicevox_core.rs");

use std::ffi::CStr;

pub type InitializeOptions = VoicevoxInitializeOptions;
pub type AudioQueryOptions = VoicevoxAudioQueryOptions;
pub type SynthesisOptions = VoicevoxSynthesisOptions;
pub type TtsOptions = VoicevoxTtsOptions;

///
/// Holds the pointer generated by the Voicevox core and performs memory management through RAII.
///
pub struct CPointerWrap<T> {
    bytes: *mut T,
    length: usize,
    free_fn: fn(*mut T),
}

impl<T> CPointerWrap<T> {
    pub fn new(bytes: *mut T, length: usize, free_fn: fn(*mut T)) -> Self {
        Self {
            bytes,
            length,
            free_fn,
        }
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.bytes, self.length) }
    }
}

impl<T> Drop for CPointerWrap<T> {
    fn drop(&mut self) {
        (self.free_fn)(self.bytes);
    }
}

///
/// Holds the string generated by the Voicevox core and performs memory management through RAII.
///
pub struct CStrWrap {
    string: *mut std::os::raw::c_char,
    free_fn: fn(*mut std::os::raw::c_char),
}

impl CStrWrap {
    pub fn new(string: *mut std::os::raw::c_char, free_fn: fn(*mut std::os::raw::c_char)) -> Self {
        Self { string, free_fn }
    }

    pub fn as_str(&self) -> &str {
        unsafe { CStr::from_ptr(self.string) }.to_str().unwrap()
    }
}

impl Drop for CStrWrap {
    fn drop(&mut self) {
        (self.free_fn)(self.string);
    }
}

/// Enum that represents the result of a Voicevox operation.
#[repr(i32)]
#[derive(Debug, PartialEq, Eq)]
pub enum ResultCode {
    /// Success
    Ok = 0,
    /// Failed to load Open JTalk dictionary file
    NotLoadedOpenjtalkDictError = 1,
    /// Failed to load the model
    LoadModelError = 2,
    /// Failed to get supported device information
    GetSupportedDevicesError = 3,
    /// GPU mode is not supported
    GpuSupportError = 4,
    /// Failed to load meta information
    LoadMetasError = 5,
    /// Status is uninitialized
    UninitializedStatusError = 6,
    /// Invalid speaker ID specified
    InvalidSpeakerIdError = 7,
    /// Invalid model index specified
    InvalidModelIndexError = 8,
    /// Inference failed
    InferenceError = 9,
    /// Failed to output context labels
    ExtractFullContextLabelError = 10,
    /// Invalid UTF-8 string input
    InvalidUtf8InputError = 11,
    /// Failed to parse Aquestalk-style text
    ParseKanaError = 12,
    /// Invalid AudioQuery
    InvalidAudioQueryError = 13,
}

///
/// Hardware acceleration mode
///
#[derive(Copy, Clone)]
pub enum AccelerationMode {
    Auto = VoicevoxAccelerationMode_VOICEVOX_ACCELERATION_MODE_AUTO as isize,
    CPU = VoicevoxAccelerationMode_VOICEVOX_ACCELERATION_MODE_CPU as isize,
    GPU = VoicevoxAccelerationMode_VOICEVOX_ACCELERATION_MODE_GPU as isize,
}

///
/// Provides functionality of Voicevox Core.
///
/// # Safety
/// VoicevoxCore is not thread-safe, and should not be accessed by multiple threads simultaneously.
/// VoicevoxCore is designed to be used as a singleton instance in a process.
/// Multiple instances of VoicevoxCore should not be created within the same process.
///
pub struct VoicevoxCore;

impl VoicevoxCore {
    pub fn new(opt: InitializeOptions) -> Result<Self, ResultCode> {
        let result = unsafe { voicevox_initialize(opt) };

        match result {
            0 => Ok(Self {}),
            e => Err(unsafe { std::mem::transmute(e) }),
        }
    }

    pub fn new_from_options(
        acceleration_mode: AccelerationMode,
        cpu_num_threads: u16,
        load_all_models: bool,
        open_jtalk_dict_dir: &std::ffi::CStr,
    ) -> Result<Self, ResultCode> {
        let opt = InitializeOptions {
            acceleration_mode: acceleration_mode as i32,
            cpu_num_threads,
            load_all_models,
            open_jtalk_dict_dir: open_jtalk_dict_dir.as_ptr(),
        };

        Self::new(opt)
    }
}

impl Drop for VoicevoxCore {
    fn drop(&mut self) {
        unsafe { voicevox_finalize() };
    }
}

impl VoicevoxCore {
    pub fn make_default_initialize_options() -> InitializeOptions {
        unsafe { voicevox_make_default_initialize_options() }
    }

    pub fn make_default_tts_options() -> TtsOptions {
        unsafe { voicevox_make_default_tts_options() }
    }

    pub fn make_default_audio_query_options() -> AudioQueryOptions {
        unsafe { voicevox_make_default_audio_query_options() }
    }

    pub fn make_default_synthesis_options() -> SynthesisOptions {
        unsafe { voicevox_make_default_synthesis_options() }
    }

    pub fn get_version() -> &'static str {
        let version_ptr = unsafe { voicevox_get_version() };

        let version_cstr = unsafe { std::ffi::CStr::from_ptr(version_ptr) };

        version_cstr.to_str().unwrap()
    }

    ///
    /// Returns the metadata of all the available voice models in JSON format.
    ///
    /// # Returns
    ///
    /// Returns a string representing the metadata of all available voice models in JSON format.
    ///
    pub fn get_metas_json() -> &'static str {
        unsafe { CStr::from_ptr(voicevox_get_metas_json()).to_str().unwrap() }
    }

    ///
    /// Returns the list of devices supported by Voicevox in JSON format.
    ///
    /// # Returns
    ///
    /// Returns a string representing the list of devices supported by Voicevox in JSON format.
    ///
    pub fn get_supported_devices_json() -> &'static str {
        unsafe {
            CStr::from_ptr(voicevox_get_supported_devices_json())
                .to_str()
                .unwrap()
        }
    }

    ///
    /// Loads a model for the specified speaker ID.
    ///
    /// # Arguments
    ///
    /// * `speaker_id` - The ID of the speaker to load the model for.
    ///
    /// # Returns
    ///
    /// If the model was loaded successfully, returns `Ok(())`.
    /// If an error occurred, returns a `ResultCode` enum value representing the error.
    ///
    pub fn load_model(&self, speaker_id: u32) -> Result<(), ResultCode> {
        let result_code = unsafe { voicevox_load_model(speaker_id) };
        if result_code == ResultCode::Ok as i32 {
            Ok(())
        } else {
            Err(unsafe { std::mem::transmute(result_code) })
        }
    }

    ///
    /// Returns a boolean value indicating whether the current process is running in GPU mode.
    ///
    /// # Returns
    ///
    /// Returns `true` if the current process is running in GPU mode, and `false` otherwise.
    ///
    pub fn is_gpu_mode(&self) -> bool {
        unsafe { voicevox_is_gpu_mode() }
    }

    ///
    /// Returns a boolean value indicating whether a voice model with the specified speaker ID has been loaded.
    ///
    /// # Arguments
    ///
    /// * `speaker_id` - The ID of the speaker to check for.
    ///
    /// # Returns
    ///
    /// Returns `true` if a voice model with the specified speaker ID has been loaded, and `false` otherwise.
    ///
    pub fn is_model_loaded(&self, speaker_id: u32) -> bool {
        unsafe { voicevox_is_model_loaded(speaker_id) }
    }

    /// Predicts the duration of each frame of a phoneme vector.
    ///
    /// # Arguments
    ///
    /// * `phoneme_vector` - A slice of the phoneme vector.
    /// * `speaker_id` - The ID of the speaker to synthesize speech for.
    ///
    /// # Returns
    ///
    /// * [CPointerWrap] if the synthesis succeeds.
    /// * An error code otherwise.
    ///
    pub fn predict_duration(
        &self,
        phoneme_vector: &[i64],
        speaker_id: u32,
    ) -> Result<CPointerWrap<f32>, ResultCode> {
        let len = phoneme_vector.len();
        let ptr = phoneme_vector.as_ptr() as *mut i64;
        let mut data_length: usize = 0;
        let mut data_ptr: *mut f32 = std::ptr::null_mut();
        let result_code = unsafe {
            voicevox_predict_duration(len, ptr, speaker_id, &mut data_length, &mut data_ptr)
        };

        match result_code {
            0 => {
                let ptr_wrap = CPointerWrap::new(data_ptr, data_length, |p| unsafe {
                    voicevox_predict_duration_data_free(p)
                });
                Ok(ptr_wrap)
            }

            e => Err(unsafe { std::mem::transmute(e) }),
        }
    }

    /// Predict intonation of a voice segment from given phoneme vectors.
    ///
    /// This function predicts intonation for a voice segment from given phoneme vectors.
    /// The predicted data is returned as a dynamically allocated array of `f32`.
    ///
    /// # Arguments
    ///
    /// * `vowel_phoneme_vector` - A slice of `i64` representing vowel phoneme vectors.
    /// * `consonant_phoneme_vector` - A slice of `i64` representing consonant phoneme vectors.
    /// * `start_accent_vector` - A slice of `i64` representing start accent vectors.
    /// * `end_accent_vector` - A slice of `i64` representing end accent vectors.
    /// * `start_accent_phrase_vector` - A slice of `i64` representing start accent phrase vectors.
    /// * `end_accent_phrase_vector` - A slice of `i64` representing end accent phrase vectors.
    /// * `speaker_id` - An `u32` representing speaker ID.
    ///
    /// # Returns
    ///
    /// On success, returns `Ok` with a `CPointerWrap<f32>` containing the predicted intonation data.
    ///
    /// On failure, returns `Err` with a `ResultCode` indicating the reason of failure.
    #[allow(clippy::too_many_arguments)]
    pub fn predict_intonation(
        &self,
        vowel_phoneme_vector: &[i64],
        consonant_phoneme_vector: &[i64],
        start_accent_vector: &[i64],
        end_accent_vector: &[i64],
        start_accent_phrase_vector: &[i64],
        end_accent_phrase_vector: &[i64],
        speaker_id: u32,
    ) -> Result<CPointerWrap<f32>, ResultCode> {
        let length = vowel_phoneme_vector.len();
        let vowel_ptr = vowel_phoneme_vector.as_ptr() as *mut i64;
        let consonant_ptr = consonant_phoneme_vector.as_ptr() as *mut i64;
        let start_accent_ptr = start_accent_vector.as_ptr() as *mut i64;
        let end_accent_ptr = end_accent_vector.as_ptr() as *mut i64;
        let start_accent_phrase_ptr = start_accent_phrase_vector.as_ptr() as *mut i64;
        let end_accent_phrase_ptr = end_accent_phrase_vector.as_ptr() as *mut i64;
        let mut output_predict_intonation_data_length: usize = 0;
        let mut output_predict_intonation_data: *mut f32 = std::ptr::null_mut();

        let result_code = unsafe {
            voicevox_predict_intonation(
                length,
                vowel_ptr,
                consonant_ptr,
                start_accent_ptr,
                end_accent_ptr,
                start_accent_phrase_ptr,
                end_accent_phrase_ptr,
                speaker_id,
                &mut output_predict_intonation_data_length,
                &mut output_predict_intonation_data,
            )
        };

        match result_code {
            0 => {
                let ptr_wrap = CPointerWrap::new(
                    output_predict_intonation_data,
                    output_predict_intonation_data_length,
                    |p| unsafe { voicevox_predict_intonation_data_free(p) },
                );
                Ok(ptr_wrap)
            }
            e => Err(unsafe { std::mem::transmute(e) }),
        }
    }

    ///
    /// Decode from phoneme and F0 vectors
    ///
    /// # Arguments
    /// - `phoneme_vectors`: Phoneme vector
    /// - `f0`: F0 vector
    /// - `speaker_id`: Speaker ID
    ///
    /// # Returns
    /// - If successful, returns a Result wrapped around a CPointerWrap object that holds the synthesized audio waveform data.
    /// - If unsuccessful, returns a Result wrapped around a ResultCode.
    ///
    pub fn decode(
        &self,
        phoneme_vectors: &[f32],
        f0: &[f32],
        speaker_id: u32,
    ) -> Result<CPointerWrap<f32>, ResultCode> {
        let phoneme_size = phoneme_vectors.len();
        let phoneme_ptr = phoneme_vectors.as_ptr() as *mut f32;
        let f0_size = f0.len();
        let f0_ptr = f0.as_ptr() as *mut f32;

        let mut data_length: usize = 0;
        let mut data_ptr: *mut f32 = std::ptr::null_mut();

        let result_code = unsafe {
            voicevox_decode(
                phoneme_size,
                f0_size / phoneme_size,
                phoneme_ptr,
                f0_ptr,
                speaker_id,
                &mut data_length,
                &mut data_ptr,
            )
        };

        match result_code {
            0 => {
                let ptr_wrap = CPointerWrap::new(data_ptr, data_length, |p| unsafe {
                    voicevox_decode_data_free(p)
                });
                Ok(ptr_wrap)
            }

            e => Err(unsafe { std::mem::transmute(e) }),
        }
    }

    /// Synthesis speech from a Voicevox audio query.
    ///
    /// # Arguments
    ///
    /// * `audio_query`: A `&str` containing the Voicevox audio query in JSON format.
    /// * `speaker_id`: An unsigned 32-bit integer representing the speaker ID to be used for the synthesis.
    /// * `options`: A struct of type `VoicevoxSynthesisOptions` representing the synthesis options.
    ///
    /// # Returns
    ///
    /// * `CPointerWrap<u8>` containing the synthesized audio in WAV format if synthesis succeeds.
    /// * An error code otherwise.
    pub fn synthesis(
        &self,
        audio_query: &str,
        speaker_id: u32,
        options: SynthesisOptions,
    ) -> Result<CPointerWrap<u8>, ResultCode> {
        let audio_query_c_str = std::ffi::CString::new(audio_query).unwrap();
        let mut output_wav_ptr: *mut u8 = std::ptr::null_mut();
        let mut output_wav_length: usize = 0;

        let result_code = unsafe {
            voicevox_synthesis(
                audio_query_c_str.as_ptr(),
                speaker_id,
                options,
                &mut output_wav_length,
                &mut output_wav_ptr,
            )
        };

        match result_code {
            0 => {
                let wav = CPointerWrap::<u8>::new(output_wav_ptr, output_wav_length, |p| unsafe {
                    voicevox_wav_free(p)
                });
                Ok(wav)
            }
            e => Err(unsafe { std::mem::transmute(e) }),
        }
    }

    /// Sends a text query to Voicevox API to retrieve audio query information.
    ///
    /// # Arguments
    ///
    /// * text - A string slice containing the input text to be synthesized.
    /// * speaker_id - An unsigned 32-bit integer representing the speaker ID to be used for the synthesis.
    /// * options - A struct of type VoicevoxAudioQueryOptions representing the synthesis options.
    ///
    /// # Returns
    ///
    /// * audio query formatted in json format if the synthesis succeeds.
    /// * An error code otherwise.
    ///
    pub fn audio_query(
        &self,
        text: &str,
        speaker_id: u32,
        options: AudioQueryOptions,
    ) -> Result<CStrWrap, ResultCode> {
        let c_str = std::ffi::CString::new(text).unwrap();
        let mut output_ptr: *mut std::os::raw::c_char = std::ptr::null_mut();
        let result_code =
            unsafe { voicevox_audio_query(c_str.as_ptr(), speaker_id, options, &mut output_ptr) };
        match result_code {
            0 => {
                let ptr_wrap =
                    CStrWrap::new(output_ptr, |p| unsafe { voicevox_audio_query_json_free(p) });
                Ok(ptr_wrap)
            }

            e => Err(unsafe { std::mem::transmute(e) }),
        }
    }

    /// This function generates a WAV file with the result of text-to-speech synthesis using Voicevox Core.
    /// This is simple version of [VoicevoxCore::tts].
    ///
    /// # Arguments
    ///
    /// * `text` - The text to be synthesized.
    /// * `speaker_id` - The ID of the speaker to be used for the synthesis.
    ///
    /// # Returns
    ///
    /// * [CPointerWrap] if the synthesis succeeds.
    /// * An error code otherwise.
    pub fn tts_simple(&self, text: &str, speaker_id: u32) -> Result<CPointerWrap<u8>, ResultCode> {
        Self::_tts(text, speaker_id, Self::make_default_tts_options())
    }

    /// This function generates a WAV file with the result of text-to-speech synthesis using Voicevox Core.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to be synthesized.
    /// * `speaker_id` - The ID of the speaker to be used for the synthesis.
    /// * `options` - The options for the synthesis.
    ///
    /// # Returns
    ///
    /// * [CPointerWrap] if the synthesis succeeds.
    /// * An error code otherwise.
    pub fn tts(
        &self,
        text: &str,
        speaker_id: u32,
        options: TtsOptions,
    ) -> Result<CPointerWrap<u8>, ResultCode> {
        Self::_tts(text, speaker_id, options)
    }

    fn _tts(
        text: &str,
        speaker_id: u32,
        options: TtsOptions,
    ) -> Result<CPointerWrap<u8>, ResultCode> {
        let c_str = std::ffi::CString::new(text).unwrap();
        let mut out_length: usize = 0;
        let mut out_wav: *mut u8 = std::ptr::null_mut();

        let result = unsafe {
            voicevox_tts(
                c_str.as_ptr(),
                speaker_id,
                options,
                &mut out_length,
                &mut out_wav,
            )
        };

        match result {
            0 => {
                let wav = CPointerWrap::<u8>::new(out_wav, out_length, |p| unsafe {
                    voicevox_wav_free(p)
                });
                Ok(wav)
            }

            e => Err(unsafe { std::mem::transmute(e) }),
        }
    }

    /// Converts a ResultCode to an error message.
    ///
    /// # Arguments
    ///
    /// * result_code - A ResultCode to convert.
    ///
    pub fn error_result_to_message(result_code: ResultCode) -> &'static str {
        unsafe {
            let message = voicevox_error_result_to_message(result_code as i32);
            CStr::from_ptr(message).to_str().unwrap()
        }
    }
}
