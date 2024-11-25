use bon::Builder;
use chrono::NaiveDate;

use crate::image::Depth;

/// Raw image header, holding all possible records and fields for BSB/KAP image files
///
/// ## Note
///
/// As with the other types exposed by the [`super::raw`] module, the validity of this type must be
/// guaranteed by the user. The only non-optional parameters in [`ImageHeader`] are:
///
/// 1. [`ImageHeader::general_parameters`] must have [`GeneralParameters::image_width_height`]
/// 2. [`ImageHeader::ifm`] must have [`Depth`]
///
// See the research materials [readme](../../../../research/readme.md) for more details
#[derive(Default, Debug, PartialEq, PartialOrd, Builder)]
#[non_exhaustive]
pub struct ImageHeader {
    /// Comments
    pub comments: Option<Vec<String>>,

    /// Record identifier: CRR
    pub copyright_record: Option<String>,

    /// Record identifier: VER
    ///
    /// Version number of BSB format e.g. 1, 2.0, 3.0, 3.07, 4.0
    pub version: Option<f32>,

    /// Record identifier: BSB (or NOS for older GEO/NOS or GEO/NO1 files)
    ///  
    /// Field definitions:
    /// ```"not rust"
    /// RA=width,height - width and height of raster image data in pixels
    /// NA=Name given to the BSB chart (can represent more than one .KAP)
    /// NU=Number of chart (especially when more than one chart is grouped or tiled together)
    /// DU=Drawing Units in pixels/inch (same as DPI resolution) e.g. 50, 150, 175, 254, 300   
    /// ```
    ///  
    pub general_parameters: GeneralParameters,

    /// Record identifier: KNP
    ///
    /// Field definitions:
    /// ```"not rust"
    /// SC=Scale e.g. 25000
    /// GD=Geodetic Datum e.g. NAD83, WGS84
    /// PR=Projection e.g. LAMBERT CONFORMAL CONIC, MERCATOR
    /// PP=Projection Parameter (value depends upon Projection) e.g. 135.0
    /// PI=? e.g. 0.0, 0.033333, 0.083333, 2.0
    /// SP=?
    /// SK=Skew angle? e.g. 0.0
    /// TA=? e.g. 90
    /// UN=Units (for DX, DY and others) e.g. METRES, FATHOMS
    /// SD=Sounding Datum e.g. MEAN LOWER LOW WATER, HHWLT
    /// DX=distance (approx.) covered by one pixel in X direction
    /// DY=distance (approx.) covered by one pixel in Y direction   
    /// ```
    ///
    pub detailed_parameters: Option<DetailedParameters>,

    /// Record identifier: KNQ
    ///
    /// Field definitions:
    /// ```"not rust"
    /// P1=...,P2=...
    /// P3=...,P4=...
    /// P5=...,P6=...
    /// ```
    pub additional_parameters: Option<AdditionalParameters>,

    /// Record identifier: CED
    pub ced: Option<ChartEditionParameters>,

    /// Record identifier: NTM
    pub ntm: Option<NTMRecord>,

    /// Offset values section
    ///
    /// Record identifier: OST
    ///
    /// Represents the number of image rows per entry in the index table (i.e. 1)
    pub ost: Option<usize>,

    /// Record identifier: IFM
    ///
    /// Compression type, or depth of the colormap (bits per pixel). BSB supports 1 through 7 (2 through 127 max colors)
    pub ifm: Depth,

    /// Record identifier: RGB
    ///
    /// RGB Default color palette
    ///
    /// Entries in the raster colormap of the form index,red,green,blue (index 0 is not used in BSB)
    ///
    /// Corresponds to [`super::ColorPalette::Rgb`]
    pub rgb: Option<Vec<(u8, u8, u8)>>,

    /// Record identifier: DAY
    ///
    /// Corresponds to [`super::ColorPalette::Day`]
    pub day: Option<Vec<(u8, u8, u8)>>,

    /// Record identifier: DSK
    ///
    /// Corresponds to [`super::ColorPalette::Dsk`]
    pub dsk: Option<Vec<(u8, u8, u8)>>,

    /// Record identifier: NGT
    /// Night Color palette
    ///
    /// Corresponds to [`super::ColorPalette::Ngt`]
    pub ngt: Option<Vec<(u8, u8, u8)>>,

    /// Record identifier: NGR
    ///
    /// Corresponds to [`super::ColorPalette::Ngr`]
    pub ngr: Option<Vec<(u8, u8, u8)>>,

    /// Record identifier: GRY
    ///
    /// Corresponds to [`super::ColorPalette::Gry`]
    pub gry: Option<Vec<(u8, u8, u8)>>,

    /// Optional palette
    ///
    /// Record identifier: PRC
    ///
    /// Corresponds to [`super::ColorPalette::Prc`]
    pub prc: Option<Vec<(u8, u8, u8)>>,

    /// Optional Grey palette
    ///
    /// Record identifier: PRG
    ///
    /// Corresponds to [`super::ColorPalette::Prg`]
    pub prg: Option<Vec<(u8, u8, u8)>>,

    /// Record identifier: REF
    ///
    /// REF Mechanism to allow geographical positions to be converted to RNC (pixel) coordinates (registration reference points)
    pub reference_point_record: Option<Vec<Ref>>,

    /// Record identifier: CPH
    ///
    /// Phase shift value
    pub phase_shift: Option<f64>,

    /// Record identifier: WPX
    ///
    /// Polynomial L to X
    pub wpx: Option<Polynomial>,

    /// Record identifier: PWX
    ///
    /// Polynomial X to L
    pub pwx: Option<Polynomial>,

    /// Record identifier: WPY
    ///
    /// Polynomial L to Y
    pub wpy: Option<Polynomial>,

    /// Record identifier: PWY
    ///
    /// Polynomial Y to L
    pub pwy: Option<Polynomial>,

    /// Record identifier: ERR
    ///
    /// ERR Error record
    pub err: Option<Vec<[f64; 4]>>,

    /// Record identifier: PLY
    ///
    /// Border polygon of the map within the raster image, given in chart datum lat/long
    pub ply: Option<Vec<(f64, f64)>>,

    /// Record identifier: DTM
    ///
    /// Data Shift Record
    ///
    /// Datum's northing and easting in floating point seconds (appears to be optional for many charts)
    pub dtm: Option<(f64, f64)>,
}

/// Represents a Polynomial used to map L <-> X || L <-> Y
#[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
pub struct Polynomial {
    /// values appear to be 1-4 going from 1=SW 2=NW 3=NE 4=SE
    // TODO: unclear what this is - perhaps the corner of the chart?
    pub corner: usize,

    /// The coordinates used by the polynomial
    pub poly: [f64; 6],
}

impl Polynomial {
    pub(crate) const fn new(corner: usize, poly: [f64; 6]) -> Self {
        Self { corner, poly }
    }
}

impl ImageHeader {
    pub(crate) fn empty() -> Self {
        Self::default()
    }

    pub(crate) const fn width(&self) -> u16 {
        self.general_parameters.image_width_height.0
    }

    pub(crate) const fn height(&self) -> u16 {
        self.general_parameters.image_width_height.1
    }
}

/// Record identifier: BSB
#[derive(Builder, Default, Debug, Clone, Eq, PartialEq, PartialOrd)]
#[non_exhaustive]
pub struct GeneralParameters {
    /// Field identifier: NA
    /// RNC name
    pub chart_name: Option<String>,

    /// Field identifier: NU
    /// RNC number
    pub chart_number: Option<String>,

    /// Field identifier: RA
    pub image_width_height: (u16, u16),

    /// Field identifier: DU
    /// Pixel resolution of the image
    pub drawing_units: Option<usize>,
}

/// Record identifier: KNP
#[derive(Builder, Default, Debug, Clone, PartialEq, PartialOrd)]
#[non_exhaustive]
pub struct DetailedParameters {
    /// Field identifier: SC
    /// Chart scale
    pub chart_scale: Option<usize>,

    /// Field identifier: GD
    /// Geodetic Datum name
    pub geodetic_datum_name: Option<String>,

    /// Field identifier: PR
    /// Projection name
    pub projection_name: Option<String>,

    /// Field identifier: PP
    pub projection_parameter: Option<f32>,

    /// Field identifier: PI
    pub projection_interval: Option<f32>,

    /// Field identifier: SP
    // Unknown
    pub sp: Option<String>,

    /// Field identifier: SK
    /// Orientation of the north
    /// Skew Angel in the original [?sic]
    pub skew_angle: Option<f32>,

    /// Field identifier: TA
    pub text_angle: Option<f32>,

    /// Field identifier: UN
    /// Depth and height units
    pub depth_units: Option<String>,

    /// Field identifier: SD
    /// Vertical datums
    pub sounding_datum: Option<String>,

    /// Field identifier: DX
    pub x_resolution: Option<f32>,

    /// Field identifier: DY
    pub y_resolution: Option<f32>,
}

/// Record identifier: KNQ
#[derive(Builder, Default, Debug, Clone, PartialEq, PartialOrd)]
#[non_exhaustive]
pub struct AdditionalParameters {
    /// Optional * P1=?
    pub p1: Option<String>,
    /// P2 - the same as KNP PP for Mercator and Transverse Mercator projection
    pub p2: Option<f32>,
    /// P3= scale factor for Transverse Mercator; 1st standard parallel for lambert conic
    pub p3: Option<String>,
    /// P4= 0 for Transverse Mercator; 2nd standard parallel for lambert conic
    pub p4: Option<String>,
    /// P5= Central meridian for Transverse Mercator and lambert conic
    pub p5: Option<String>,
    /// P6= origin of latitude for Transverse Mercator and lambert conic
    pub p6: Option<String>,
    /// P7 = `+x_0` for Transverse Mercator and lambert conic
    pub p7: Option<String>,
    /// P8 = `+y_0` for Transverse Mercator and lambert conic
    pub p8: Option<String>,
    /// Unknown
    // TODO: often seen with the following value: 'EC=?'
    pub ec: Option<String>,
    /// Unknown
    //  NOTE: not same GD between KNP and KNQ
    pub gd: Option<String>,
    /// Unknown
    pub vc: Option<String>,
    /// Unknown
    pub sc: Option<String>,
    /// Unknown
    pub gc: Option<String>,
    /// Unknown
    pub rm: Option<String>,
    /// PC=?. Set to TC for Transverse Mercator
    pub pc: Option<String>,
}

/// Record identifier: CED
#[derive(Builder, Default, Debug, Clone, Eq, PartialEq, PartialOrd)]
#[non_exhaustive]
pub struct ChartEditionParameters {
    /// Field identifier: SE
    pub source_edition: Option<usize>,
    /// Field identifier: RE
    pub raster_edition: Option<usize>,
    /// Field identifier: ED
    pub edition_date: Option<NaiveDate>,
}

/// Record identifier: NTM
#[derive(Builder, Default, Debug, Clone, PartialEq, PartialOrd)]
#[non_exhaustive]
pub struct NTMRecord {
    /// Field identifier: NE
    /// NTM edition
    pub edition: Option<f32>,
    /// Field identifier: ND
    /// NTM date
    pub date: Option<NaiveDate>,
    /// Field identifier: BF
    /// Base flag
    pub base_flag: Option<String>,
    /// Field identifier: BD
    /// ADN Record
    pub adn_record: Option<NaiveDate>,
}

/// Record identifier: REF
///
/// Used to map pixel coordinates to geographical coordinates
#[derive(Builder, Default, Debug, Clone, PartialEq, PartialOrd)]
#[non_exhaustive]
pub struct Ref {
    /// The given pixel coordinates
    pub pixels: (usize, usize),
    /// The corresponding geographical coordinates
    pub coords: (f64, f64),
}
