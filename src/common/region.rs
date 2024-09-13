use scylla::{cql_to_rust::{FromCqlVal, FromCqlValError}, frame::response::result::{ColumnType, CqlValue}, serialize::{value::SerializeValue, writers::WrittenCellProof, CellWriter, SerializationError}};
use serde::{de, Deserialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Region {
  Afghanistan = 0,
  Albania = 1,
  Algeria = 2,
  Andorra = 3,
  Angola = 4,
  AntiguaAndBarbuda = 5,
  Argentina = 6,
  Armenia = 7,
  Australia = 8,
  Austria = 9,
  Azerbaijan = 10,
  Bahamas = 11,
  Bahrain = 12,
  Bangladesh = 13,
  Barbados = 14,
  Belarus = 15,
  Belgium = 16,
  Belize = 17,
  Benin = 18,
  Bhutan = 19,
  Bolivia = 20,
  BosniaAndHerzegovina = 21,
  Botswana = 22,
  Brazil = 23,
  BruneiDarussalam = 24,
  Bulgaria = 25,
  BurkinaFaso = 26,
  Burundi = 27,
  CaboVerde = 28,
  Cambodia = 29,
  Cameroon = 30,
  Canada = 31,
  CentralAfricanRepublic = 32,
  Chad = 33,
  Chile = 34,
  China = 35,
  Colombia = 36,
  Comoros = 37,
  CookIslands = 38,
  RepublicOfTheCongo = 39,
  CostaRica = 40,
  CoteDIvoire = 41,
  Croatia = 42,
  Cuba = 43,
  Cyprus = 44,
  CzechRepublic = 45,
  DemocraticRepublicOfTheCongo = 46,
  Denmark = 47,
  Djibouti = 48,
  Dominica = 49,
  DominicanRepublic = 50,
  Ecuador = 51,
  Egypt = 52,
  ElSalvador = 53,
  EquatorialGuinea = 54,
  Eritrea = 55,
  Estonia = 56,
  Eswatini = 57,
  Ethiopia = 58,
  Fiji = 59,
  Finland = 60,
  France = 61,
  Gabon = 62,
  Gambia = 63,
  Georgia = 64,
  Germany = 65,
  Ghana = 66,
  Greece = 67,
  Grenada = 68,
  Guatemala = 69,
  Guinea = 70,
  GuineaBissau = 71,
  Guyana = 72,
  Haiti = 73,
  Honduras = 74,
  HongKong = 75,
  Hungary = 76,
  Iceland = 77,
  India = 78,
  Indonesia = 79,
  Iran = 80,
  Iraq = 81,
  Ireland = 82,
  Israel = 83,
  Italy = 84,
  Jamaica = 85,
  Japan = 86,
  Jordan = 87,
  Kazakhstan = 88,
  Kenya = 89,
  Kiribati = 90,
  Kuwait = 91,
  Kyrgyzstan = 92,
  LaoPeoplesDemocraticRepublic = 93,
  Latvia = 94,
  Lebanon = 95,
  Lesotho = 96,
  Liberia = 97,
  Libya = 98,
  Liechtenstein = 99,
  Lithuania = 100,
  Luxembourg = 101,
  Madagascar = 102,
  Malawi = 103,
  Malaysia = 104,
  Maldives = 105,
  Mali = 106,
  Malta = 107,
  MarshallIslands = 108,
  Mauritania = 109,
  Mauritius = 110,
  Mexico = 111,
  MicronesiaFederatedStatesOf = 112,
  Monaco = 113,
  Mongolia = 114,
  Montenegro = 115,
  Morocco = 116,
  Mozambique = 117,
  Myanmar = 118,
  Namibia = 119,
  Nauru = 120,
  Nepal = 121,
  Netherlands = 122,
  NewZealand = 123,
  Nicaragua = 124,
  Niger = 125,
  Nigeria = 126,
  Niue = 127,
  NorthMacedonia = 128,
  Norway = 129,
  Oman = 130,
  Pakistan = 131,
  Palau = 132,
  Panama = 133,
  PapuaNewGuinea = 134,
  Paraguay = 135,
  Peru = 136,
  Philippines = 137,
  Poland = 138,
  Portugal = 139,
  Qatar = 140,
  RepublicOfKorea = 141,
  RepublicOfKosovo = 142,
  RepublicOfMoldova = 143,
  Romania = 144,
  RussianFederation = 145,
  Rwanda = 146,
  SaintKittsAndNevis = 147,
  SaintLucia = 148,
  SaintVincentAndTheGrenadines = 149,
  Samoa = 150,
  SanMarino = 151,
  SaoTomeAndPrincipe = 152,
  SaudiArabia = 153,
  Senegal = 154,
  Serbia = 155,
  Seychelles = 156,
  SierraLeone = 157,
  Singapore = 158,
  Slovakia = 159,
  Slovenia = 160,
  SolomonIslands = 161,
  Somalia = 162,
  SouthAfrica = 163,
  SouthSudan = 164,
  Spain = 165,
  SriLanka = 166,
  Sudan = 167,
  Suriname = 168,
  Sweden = 169,
  Switzerland = 170,
  Syria = 171,
  Taiwan = 172,
  Tajikistan = 173,
  Thailand = 174,
  TimorLeste = 175,
  Togo = 176,
  Tonga = 177,
  TrinidadAndTobago = 178,
  Tunisia = 179,
  Turkey = 180,
  Turkmenistan = 181,
  Tuvalu = 182,
  Uganda = 183,
  Ukraine = 184,
  UnitedArabEmirates = 185,
  UnitedKingdom = 186,
  UnitedRepublicOfTanzania = 187,
  UnitedStatesOfAmerica = 188,
  Uruguay = 189,
  Uzbekistan = 190,
  Vanuatu = 191,
  Vatican = 192,
  Venezuela = 193,
  VietNam = 194,
  Yemen = 195,
  Zambia = 196,
  Zimbabwe = 197,
}

impl From<Region> for u8 {
  fn from(value: Region) -> Self {
      value as u8
  }
}

impl From<Region> for i8 {
  fn from(value: Region) -> Self {
    u8::from(value) as i8
  }
}

#[derive(Debug, PartialEq, Error)]
#[error("有効な地域ではありません")]
pub struct ParseRegionError;

impl TryFrom<u8> for Region {
  type Error = ParseRegionError;

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    let region = match value {
      0 => Region::Afghanistan,
      1 => Region::Albania,
      2 => Region::Algeria,
      3 => Region::Andorra,
      4 => Region::Angola,
      5 => Region::AntiguaAndBarbuda,
      6 => Region::Argentina,
      7 => Region::Armenia,
      8 => Region::Australia,
      9 => Region::Austria,
      10 => Region::Azerbaijan,
      11 => Region::Bahamas,
      12 => Region::Bahrain,
      13 => Region::Bangladesh,
      14 => Region::Barbados,
      15 => Region::Belarus,
      16 => Region::Belgium,
      17 => Region::Belize,
      18 => Region::Benin,
      19 => Region::Bhutan,
      20 => Region::Bolivia,
      21 => Region::BosniaAndHerzegovina,
      22 => Region::Botswana,
      23 => Region::Brazil,
      24 => Region::BruneiDarussalam,
      25 => Region::Bulgaria,
      26 => Region::BurkinaFaso,
      27 => Region::Burundi,
      28 => Region::CaboVerde,
      29 => Region::Cambodia,
      30 => Region::Cameroon,
      31 => Region::Canada,
      32 => Region::CentralAfricanRepublic,
      33 => Region::Chad,
      34 => Region::Chile,
      35 => Region::China,
      36 => Region::Colombia,
      37 => Region::Comoros,
      38 => Region::CookIslands,
      39 => Region::RepublicOfTheCongo,
      40 => Region::CostaRica,
      41 => Region::CoteDIvoire,
      42 => Region::Croatia,
      43 => Region::Cuba,
      44 => Region::Cyprus,
      45 => Region::CzechRepublic,
      46 => Region::DemocraticRepublicOfTheCongo,
      47 => Region::Denmark,
      48 => Region::Djibouti,
      49 => Region::Dominica,
      50 => Region::DominicanRepublic,
      51 => Region::Ecuador,
      52 => Region::Egypt,
      53 => Region::ElSalvador,
      54 => Region::EquatorialGuinea,
      55 => Region::Eritrea,
      56 => Region::Estonia,
      57 => Region::Eswatini,
      58 => Region::Ethiopia,
      59 => Region::Fiji,
      60 => Region::Finland,
      61 => Region::France,
      62 => Region::Gabon,
      63 => Region::Gambia,
      64 => Region::Georgia,
      65 => Region::Germany,
      66 => Region::Ghana,
      67 => Region::Greece,
      68 => Region::Grenada,
      69 => Region::Guatemala,
      70 => Region::Guinea,
      71 => Region::GuineaBissau,
      72 => Region::Guyana,
      73 => Region::Haiti,
      74 => Region::Honduras,
      75 => Region::HongKong,
      76 => Region::Hungary,
      77 => Region::Iceland,
      78 => Region::India,
      79 => Region::Indonesia,
      80 => Region::Iran,
      81 => Region::Iraq,
      82 => Region::Ireland,
      83 => Region::Israel,
      84 => Region::Italy,
      85 => Region::Jamaica,
      86 => Region::Japan,
      87 => Region::Jordan,
      88 => Region::Kazakhstan,
      89 => Region::Kenya,
      90 => Region::Kiribati,
      91 => Region::Kuwait,
      92 => Region::Kyrgyzstan,
      93 => Region::LaoPeoplesDemocraticRepublic,
      94 => Region::Latvia,
      95 => Region::Lebanon,
      96 => Region::Lesotho,
      97 => Region::Liberia,
      98 => Region::Libya,
      99 => Region::Liechtenstein,
      100 => Region::Lithuania,
      101 => Region::Luxembourg,
      102 => Region::Madagascar,
      103 => Region::Malawi,
      104 => Region::Malaysia,
      105 => Region::Maldives,
      106 => Region::Mali,
      107 => Region::Malta,
      108 => Region::MarshallIslands,
      109 => Region::Mauritania,
      110 => Region::Mauritius,
      111 => Region::Mexico,
      112 => Region::MicronesiaFederatedStatesOf,
      113 => Region::Monaco,
      114 => Region::Mongolia,
      115 => Region::Montenegro,
      116 => Region::Morocco,
      117 => Region::Mozambique,
      118 => Region::Myanmar,
      119 => Region::Namibia,
      120 => Region::Nauru,
      121 => Region::Nepal,
      122 => Region::Netherlands,
      123 => Region::NewZealand,
      124 => Region::Nicaragua,
      125 => Region::Niger,
      126 => Region::Nigeria,
      127 => Region::Niue,
      128 => Region::NorthMacedonia,
      129 => Region::Norway,
      130 => Region::Oman,
      131 => Region::Pakistan,
      132 => Region::Palau,
      133 => Region::Panama,
      134 => Region::PapuaNewGuinea,
      135 => Region::Paraguay,
      136 => Region::Peru,
      137 => Region::Philippines,
      138 => Region::Poland,
      139 => Region::Portugal,
      140 => Region::Qatar,
      141 => Region::RepublicOfKorea,
      142 => Region::RepublicOfKosovo,
      143 => Region::RepublicOfMoldova,
      144 => Region::Romania,
      145 => Region::RussianFederation,
      146 => Region::Rwanda,
      147 => Region::SaintKittsAndNevis,
      148 => Region::SaintLucia,
      149 => Region::SaintVincentAndTheGrenadines,
      150 => Region::Samoa,
      151 => Region::SanMarino,
      152 => Region::SaoTomeAndPrincipe,
      153 => Region::SaudiArabia,
      154 => Region::Senegal,
      155 => Region::Serbia,
      156 => Region::Seychelles,
      157 => Region::SierraLeone,
      158 => Region::Singapore,
      159 => Region::Slovakia,
      160 => Region::Slovenia,
      161 => Region::SolomonIslands,
      162 => Region::Somalia,
      163 => Region::SouthAfrica,
      164 => Region::SouthSudan,
      165 => Region::Spain,
      166 => Region::SriLanka,
      167 => Region::Sudan,
      168 => Region::Suriname,
      169 => Region::Sweden,
      170 => Region::Switzerland,
      171 => Region::Syria,
      172 => Region::Taiwan,
      173 => Region::Tajikistan,
      174 => Region::Thailand,
      175 => Region::TimorLeste,
      176 => Region::Togo,
      177 => Region::Tonga,
      178 => Region::TrinidadAndTobago,
      179 => Region::Tunisia,
      180 => Region::Turkey,
      181 => Region::Turkmenistan,
      182 => Region::Tuvalu,
      183 => Region::Uganda,
      184 => Region::Ukraine,
      185 => Region::UnitedArabEmirates,
      186 => Region::UnitedKingdom,
      187 => Region::UnitedRepublicOfTanzania,
      188 => Region::UnitedStatesOfAmerica,
      189 => Region::Uruguay,
      190 => Region::Uzbekistan,
      191 => Region::Vanuatu,
      192 => Region::Vatican,
      193 => Region::Venezuela,
      194 => Region::VietNam,
      195 => Region::Yemen,
      196 => Region::Zambia,
      197 => Region::Zimbabwe,
      _ => return Err(ParseRegionError),
    };
    Ok(region)
  }
}

impl TryFrom<i8> for Region {
  type Error = ParseRegionError;

  fn try_from(value: i8) -> Result<Self, Self::Error> {
    Region::try_from(value as u8)
  }
}

impl<'de> Deserialize<'de> for Region {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
      D: serde::Deserializer<'de>
  {
      let n: u8 = Deserialize::deserialize(deserializer)?;
      Region::try_from(n).map_err(de::Error::custom)
  }
}

impl SerializeValue for Region {
  fn serialize<'b>(&self, typ: &ColumnType, writer: CellWriter<'b>) -> Result<WrittenCellProof<'b>, SerializationError> {
    i8::from(*self).serialize(typ, writer)
  }
}

impl FromCqlVal<Option<CqlValue>> for Region {
  fn from_cql(cql_val: Option<CqlValue>) -> Result<Self, FromCqlValError> {
    i8::from_cql(cql_val).and_then(|n| Region::try_from(n).map_err(|_| FromCqlValError::BadVal))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn try_from_valid_u8() {
    for i in 0u8..=197 {
      let region = Region::try_from(i);
      assert_eq!(region.map(u8::from), Ok(i));
    }
  }

  #[test]
  fn try_from_invalid_u8() {
    for i in 198u8..=u8::MAX {
      let region = Region::try_from(i);
      assert_eq!(region.map(u8::from), Err(ParseRegionError));
    }
  }

  #[test]
  fn try_from_valid_i8() {
    for i in 0u8..=197 {
      let i = i as i8;
      let region = Region::try_from(i);
      assert_eq!(region.map(i8::from), Ok(i));
    }
  }

  #[test]
  fn try_from_invalid_i8() {
    for i in 198u8..=u8::MAX {
      let i = i as i8;
      let region = Region::try_from(i);
      assert_eq!(region.map(i8::from), Err(ParseRegionError));
    }
  }

  #[test]
  fn deserialize_valid_json() {
    let json = r#"86"#;
    let region: Region = serde_json::from_str(json).unwrap();
    assert_eq!(region, Region::Japan);
  }
  
  #[test]
  fn deserialize_invalid_json() {
    let json = r#"-1"#;
    let region = serde_json::from_str::<Region>(json);
    assert!(region.is_err());
  }
}
