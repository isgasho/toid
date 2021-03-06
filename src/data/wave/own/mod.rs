use std::fs;
use std::io::Write;

use nom::number::streaming::{le_i16, le_i24};
use nom::IResult;

use super::parsed;

#[derive(Clone, Debug)]
pub enum Data {
    Monoral(Vec<f32>),
    Stereo((Vec<f32>, Vec<f32>)),
}

impl Data {
    fn get_chunnel_size(&self) -> usize {
        match self {
            Data::Monoral(_) => 1,
            Data::Stereo(_) => 2,
        }
    }
}

#[derive(Clone)]
pub struct Wave {
    pub data: Data,
    pub sample_num: usize,
    pub sample_rate: f32,
}

impl Wave {
    pub fn parse(i: &[u8]) -> Result<Self, String> {
        let parsed_wave = parsed::Wave::parse(i)?;
        Self::parsed_wave_to_own_wave(parsed_wave)
    }

    pub fn get_samples(&self, start: usize, end: usize) -> Result<(Vec<f32>, Vec<f32>), String> {
        let mut left_sample = Vec::new();
        let mut right_sample = Vec::new();

        let size = end - start;

        let start = std::cmp::min(start, self.sample_num);
        let end = std::cmp::min(end, self.sample_num);

        match &self.data {
            Data::Monoral(data) => {
                left_sample.resize(end - start, 0.0);
                right_sample.resize(end - start, 0.0);
                left_sample.copy_from_slice(data.split_at(start).1.split_at(end - start).0);
                right_sample.copy_from_slice(data.split_at(start).1.split_at(end - start).0);
                left_sample.resize(size, 0.0);
                right_sample.resize(size, 0.0);
            }
            Data::Stereo((left_data, right_data)) => {
                left_sample.resize(end - start, 0.0);
                right_sample.resize(end - start, 0.0);
                left_sample.copy_from_slice(left_data.split_at(start).1.split_at(end - start).0);
                right_sample.copy_from_slice(right_data.split_at(start).1.split_at(end - start).0);
                left_sample.resize(size, 0.0);
                right_sample.resize(size, 0.0);
            }
        };

        Ok((left_sample, right_sample))
    }

    fn parsed_wave_to_own_wave(parsed_wave: parsed::Wave) -> Result<Wave, String> {
        match parsed_wave.format.channels {
            1 => {
                let data = Self::parse_monoral_data(
                    parsed_wave.data.data.as_slice(),
                    parsed_wave.data.data.len() / (parsed_wave.format.bitswidth as usize / 8),
                    parsed_wave.format.bitswidth as usize,
                )
                .unwrap()
                .1;
                Ok(Wave {
                    data,
                    sample_num: parsed_wave.data.data.len()
                        / (parsed_wave.format.bitswidth as usize / 8),
                    sample_rate: parsed_wave.format.samplerate as f32,
                })
            }
            2 => {
                let data = Self::parse_stereo_data(
                    parsed_wave.data.data.as_slice(),
                    parsed_wave.data.data.len() / (parsed_wave.format.bitswidth as usize / 8) / 2,
                    parsed_wave.format.bitswidth as usize,
                )
                .unwrap()
                .1;
                Ok(Wave {
                    data,
                    sample_num: parsed_wave.data.data.len()
                        / (parsed_wave.format.bitswidth as usize / 8)
                        / 2,
                    sample_rate: parsed_wave.format.samplerate as f32,
                })
            }
            _ => Err("invalid channel".to_string()),
        }
    }

    fn parse_monoral_data(i: &[u8], sample_num: usize, bit_num: usize) -> IResult<&[u8], Data> {
        let mut data = vec![];
        data.resize(sample_num, 0.0);
        let mut i = i;
        match bit_num {
            16 => {
                for sample_idx in 0..sample_num {
                    let ret = le_i16(i)?;
                    i = ret.0;
                    data[sample_idx] = ret.1 as f32 / i16::MAX as f32;
                }
            }
            24 => {
                for sample_idx in 0..sample_num {
                    let ret = le_i24(i)?;
                    i = ret.0;
                    data[sample_idx] = ret.1 as f32 / (i16::MAX as i32 * u8::MAX as i32) as f32;
                }
            }
            _ => {
                return Err(nom::Err::Error((i, nom::error::ErrorKind::NoneOf)));
            }
        }
        Ok((i, Data::Monoral(data)))
    }

    fn parse_stereo_data(i: &[u8], sample_num: usize, bit_num: usize) -> IResult<&[u8], Data> {
        let mut left_data = vec![];
        let mut right_data = vec![];
        left_data.resize(sample_num, 0.0);
        right_data.resize(sample_num, 0.0);
        let mut i = i;
        match bit_num {
            16 => {
                for sample_idx in 0..sample_num {
                    let ret = le_i16(i)?;
                    i = ret.0;
                    left_data[sample_idx] = ret.1 as f32 / i16::MAX as f32;
                    let ret = le_i16(i)?;
                    i = ret.0;
                    right_data[sample_idx] = ret.1 as f32 / i16::MAX as f32;
                }
            }
            24 => {
                for sample_idx in 0..sample_num {
                    let ret = le_i24(i)?;
                    i = ret.0;
                    left_data[sample_idx] =
                        ret.1 as f32 / (i16::MAX as i32 * u8::MAX as i32) as f32;
                    let ret = le_i24(i)?;
                    i = ret.0;
                    right_data[sample_idx] =
                        ret.1 as f32 / (i16::MAX as i32 * u8::MAX as i32) as f32;
                }
            }
            _ => {
                return Err(nom::Err::Error((i, nom::error::ErrorKind::NoneOf)));
            }
        }
        Ok((i, Data::Stereo((left_data, right_data))))
    }

    fn vec_f32_access(&self, v: &Vec<f32>, i: f32) -> f32 {
        let left_idx = i as usize;
        let right_idx = left_idx + 1;
        let right_weight = i - left_idx as f32;
        let left_weight = 1.0 - right_weight;

        match (v.get(left_idx), v.get(right_idx)) {
            (Some(left_value), Some(right_value)) => {
                left_weight * left_value + right_weight * right_value
            }
            (Some(left_value), None) => left_weight * left_value,
            (None, Some(right_value)) => right_weight * right_value,
            (None, None) => 0.0,
        }
    }

    pub fn change_sample_rate(&self, sample_rate: f32) -> Self {
        let new_sample_width = self.sample_rate / sample_rate;
        let mut sample_idx: f32 = 0.0;
        let mut new_sample_num: usize = 0;

        let new_data = match &self.data {
            Data::Monoral(wave) => {
                let mut new_wave: Vec<f32> = vec![];
                while sample_idx < self.sample_num as f32 - 1.0 {
                    new_wave.push(self.vec_f32_access(wave, sample_idx));
                    sample_idx += new_sample_width;
                    new_sample_num += 1;
                }
                Data::Monoral(new_wave)
            }
            Data::Stereo((left_wave, right_wave)) => {
                let mut new_left_wave: Vec<f32> = vec![];
                let mut new_right_wave: Vec<f32> = vec![];
                while sample_idx < self.sample_num as f32 - 1.0 {
                    new_left_wave.push(self.vec_f32_access(left_wave, sample_idx));
                    new_right_wave.push(self.vec_f32_access(right_wave, sample_idx));
                    sample_idx += new_sample_width;
                    new_sample_num += 1;
                }
                Data::Stereo((new_left_wave, new_right_wave))
            }
        };

        Wave {
            data: new_data,
            sample_num: new_sample_num,
            sample_rate: sample_rate,
        }
    }

    pub fn to_riff_buffer(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = vec![];

        // head chunk
        for &v in "RIFF".as_bytes() {
            buffer.push(v);
        }

        let format_chunk_size = 4 + 4 + 2 + 2 + 4 + 4 + 2 + 2;
        let data_chunk_size = self.sample_num * 2 * self.data.get_chunnel_size() + 8;

        for &v in &((4 + format_chunk_size + data_chunk_size) as u32).to_le_bytes() {
            buffer.push(v);
        }

        for &v in "WAVE".as_bytes() {
            buffer.push(v);
        }

        // format chunk
        for &v in "fmt ".as_bytes() {
            buffer.push(v);
        }

        for &v in &(16 as u32).to_le_bytes() {
            buffer.push(v);
        }

        for &v in &(1 as u16).to_le_bytes() {
            buffer.push(v);
        }

        for &v in &(self.data.get_chunnel_size() as u16).to_le_bytes() {
            buffer.push(v);
        }

        for &v in &(self.sample_rate as u32).to_le_bytes() {
            buffer.push(v);
        }

        for &v in
            &((self.sample_rate as usize * 2 * self.data.get_chunnel_size()) as u32).to_le_bytes()
        {
            buffer.push(v);
        }

        for &v in &(2 * self.data.get_chunnel_size() as u16).to_le_bytes() {
            buffer.push(v);
        }

        for &v in &(16 as u16).to_le_bytes() {
            buffer.push(v);
        }

        // data chunk
        for &v in "data".as_bytes() {
            buffer.push(v);
        }

        for &v in &((self.sample_num * 2 * self.data.get_chunnel_size()) as u32).to_le_bytes() {
            buffer.push(v);
        }

        match &self.data {
            Data::Monoral(wave) => {
                for sample in wave.iter() {
                    for &v in &((sample * std::i16::MAX as f32) as i16).to_le_bytes() {
                        buffer.push(v);
                    }
                }
            }
            Data::Stereo((left_wave, right_wave)) => {
                for (left_sample, right_sample) in left_wave.iter().zip(right_wave.iter()) {
                    for &v in &((left_sample * std::i16::MAX as f32) as i16).to_le_bytes() {
                        buffer.push(v);
                    }
                    for &v in &((right_sample * std::i16::MAX as f32) as i16).to_le_bytes() {
                        buffer.push(v);
                    }
                }
            }
        }

        buffer
    }

    pub fn save(&self, path: String) {
        let mut file = fs::File::create(path).unwrap();
        file.write_all(self.to_riff_buffer().as_slice()).unwrap();
        file.flush().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;
    use std::io::Read;

    #[test]
    fn test_parse() {
        let paths = [
            "toid-sample-resource/samples/0_hihat_closed.wav",
            "toid-sample-resource/samples/0_snare_drum.wav",
            "toid-sample-resource/samples/3_kick_drum.wav",
            "toid-sample-resource/impulse_response/phase1_stereo.wav",
        ];
        for path in paths.iter() {
            let mut f = fs::File::open(path).map_err(|_| "file open error").unwrap();
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)
                .map_err(|_| "read error")
                .unwrap();
            let buffer = buffer.as_slice();

            Wave::parse(buffer).unwrap();
        }
    }

    #[test]
    fn test_change_sample_rate() {
        let path = "toid-sample-resource/impulse_response/phase1_stereo.wav";

        let mut f = fs::File::open(path).map_err(|_| "file open error").unwrap();
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer)
            .map_err(|_| "read error")
            .unwrap();
        let buffer = buffer.as_slice();

        let wave = Wave::parse(buffer).unwrap();

        assert_eq!(wave.sample_rate, 44100.0);

        let sample_num = wave.sample_num;

        let wave = wave.change_sample_rate(22050.0);

        assert_eq!(wave.sample_rate, 22050.0);

        assert!(wave.sample_num as f32 >= sample_num as f32 / 2.0 - 0.5);
        assert!(wave.sample_num as f32 <= sample_num as f32 / 2.0 + 0.5);
    }

    #[test]
    fn test_to_riff_buffer() {
        let path = "toid-sample-resource/samples/0_hihat_closed.wav";

        let mut f = fs::File::open(path).map_err(|_| "file open error").unwrap();
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer)
            .map_err(|_| "read error")
            .unwrap();
        let buffer = buffer.as_slice();
        let wave = Wave::parse(buffer).unwrap();

        Wave::parse(wave.to_riff_buffer().as_slice()).unwrap();
    }
}
