// Imports
#[macro_use]
extern crate clap;
use clap::Parser;
use ndarray::{ViewRepr,Dim,ArrayBase,array,Array1, Array2, Axis};
use anyhow::{Context, Result};
use serde::{Serialize,Deserialize};
use std::path::{PathBuf,Path};
use std::{io::BufReader,
					fmt,
					fs::File,
					boxed::Box,
					ops::BitAnd};

//Types

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
 		/// Turn on verbose messages
 		#[arg(short,long,default_value_t = false)]
		verbose: bool,

		/// json file to solve
		input: PathBuf,
}

#[derive(Serialize,Deserialize, Debug,Clone)]
pub struct InFile {
		/// a list of the column requierments
		columns: Vec<Vec<i8>>,
		/// a list of the row requierments
		rows: Vec<Vec<i8>>,
		/// a list of pre-filled cells
		#[serde(default)]
		filled: Vec<(i8,i8)>,
		/// a list of pre-emptied cells
		#[serde(default)]
		empty: Vec<(i8,i8)>,
}

/// a picross board in the process of being solved 
#[derive(Debug,Clone)]
struct Board {
		/// grid of cells
		grid : Array2<CellState>,
		/// height of the grid
		height: usize,
		/// width of the grid
		width: usize,
		/// a list of the column requierments
		columns: Vec<Vec<i8>>,
		/// a list of the row requierments
		rows: Vec<Vec<i8>>,
}

/// The possible states of cells in the board
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
enum CellState {
		Filled,
		Empty,
		Unknown,
}

// Functions

fn main() -> Result<()> {
		let args = Args::parse();
		// check args
		let json : InFile = serde_json::from_reader(
				BufReader::new(
				File::open(args.input.clone())
								.with_context(|| format!("unable to find file \"{}\"",args.input.display()))?
				)).with_context(|| format!{"unable to parse file \"{}\"", args.input.display()})?;
		let mut board = Board::new(json.clone())?;
		if args.verbose {
				println!("parsed \"{}\" with shape ({},{})\n column requirements: {:?}\nrow requierments: {:?}\ngrid:\n{}",
										 args.input.display(),json.columns.len(),json.rows.len(),json.columns,json.rows,board.pretty_grid_str());
				};
		// solve the board
		board.full_solve(args.verbose)?;
		// print the solution
		println!("{}",board.pretty_grid_str());
		Ok(())
}

// Impliments
impl Board {
		fn new(infile: InFile) -> Result<Self> {
				// get size of grid
				let height :usize = infile.rows.len();
				let width :usize = infile.columns.len();
				// make grid 
				let mut grid : Array2<CellState>= Array2::from_elem((width,height),CellState::Unknown);
				// fill grid with known info
				for (r,c) in infile.empty {

						grid[[r as usize,c as usize]] = CellState::Empty;
				}
				for (r,c) in infile.filled {
						grid[[r as usize,c as usize]] = CellState::Filled;
				}
				// return
				Ok(Self {
						height,width,grid,columns:infile.columns,rows:infile.rows
				})
		}
		/// returns true if board has no unknown cells
		pub fn complete(&self) -> bool {
				for ((_y,_x), value) in self.grid.indexed_iter() {
						if *value == CellState::Unknown {
								return false;
						}
				}
				return true;
		}
		/// impliments the requierments as much as possible
		/// arr: either a column or a row
		/// len: lenght of arr
		/// requierments: a list of requierd contigues filled sections with at least 1 empty cell between them
		fn impl_requierments(arr:&mut Vec<CellState>, len: usize, requirements: Vec<i8>) -> Result<()>{
				// return true if arr is valid under requierments
				// false otherwise
				fn check_requirements(arr: &Vec<CellState>,requirements:Vec<i8>) -> bool {
						let mut curr_streak = 0;
						let mut found_requierments = vec![];
						for val in arr.iter() {
								match val{
										CellState::Filled => {curr_streak += 1;},
										CellState::Empty => if curr_streak != 0 {
												found_requierments.push(curr_streak);
												curr_streak = 0;
										},
										// arr is not a valid solution if it contains any unknown cells
										CellState::Unknown => return false,
								}
								
						};
						if curr_streak != 0 {
								found_requierments.push(curr_streak);
								curr_streak = 0;
						};
						// test code for printing
						//  eprintln!("{:?} found \'{:?}\' needs \'{:?}\'. {}",arr,found_requierments,requirements.clone(),
						//  					found_requierments == requirements);
						return found_requierments == requirements;
				};
				fn advance(arr: &Vec<CellState>,
									 index: usize,
									 req_left: i8,
									 len: usize,
									 next_req: Vec<i8>,
									 // the full set of requierments for a solution
									 requirements: Vec<i8>
				)-> Vec<Vec<CellState>> {

						// if req_left if negitive this node of concederation has failed to produce a valid solution
						if req_left <0 {
								return vec![];
						};
						
						let mut total_possible:Vec<Vec<CellState>>  = vec![];
						let mut arr = arr.clone();
						let mut index = index;
						let mut req_left = req_left;
						// fill up current requierment
						while req_left > 0 {
								// eprintln!("arr={:?},index={},req_left={}",arr,index,req_left);
								// if index >= len then this node produces no valid solutions
								if index >= len {
										// eprintln!("index ({},{}) too big. req_left={}.",index,len,req_left);
										return vec![];
								};
								// if current index is empty, then this node of concederation
								// has no valid solutions. Because if the current index is empty,
								// that has already been decided and the cell cannot be anything else.
								if arr[index] == CellState::Empty {
										// eprintln!("index, {}, empty",index);
										return vec![];
								};
								arr[index] = CellState::Filled;
								req_left -= 1;
								index += 1;
						}
						// eprintln!("arr={:?},index={},req_left={}",arr,index,req_left);
						// contigues filled cells found
						let mut contig_cells : i8 = 0;
						// the last place to start a valid solution
						// let limit : usize = len-((next_req.iter().sum::<i8>() as usize)+next_req.len()) ;
						for i in (index)..len {
								match arr[i] {
										CellState::Empty => {contig_cells = 0;},
										// increse the number of cells found to be filled contiguasly
										CellState::Filled => {contig_cells+=1;},
										// add more possible solutions
										CellState::Unknown => {
												if next_req.len() >0 {
														// cases where cell is not empty
														total_possible.append(
																&mut advance(&arr.clone(),i,next_req[0]-contig_cells,len,next_req[1..].to_vec(),requirements.clone()));
												};
												// case where cell is empty
												arr[i]=CellState::Empty;
										}
								}
						}
						// return arr if valid
						if check_requirements(&arr,requirements) {
								total_possible.append(&mut vec![arr.clone()]);
						};
						total_possible
				};
				if requirements.len() >0 {
						// binary and the total possible together
						let mut total_possible:Vec<Vec<CellState>>  = advance(&arr,0,0,len,requirements.clone(),requirements.clone());
						if total_possible.len() == 0 {
								return Ok(());
						};
						// eprintln!("{:#?}",total_possible);
						let default_arr : Vec<CellState> = vec![CellState::Unknown;len];
						*arr = total_possible.get(1..).unwrap_or(&vec![]).iter().fold(
								total_possible.get(0).unwrap_or(&default_arr).to_vec(),
								|a,b| {
										let x= a.iter().zip(b.iter()).map(
												|(c,d)| *c & *d).collect();
										// eprint!(" {:?} ",x);
										x
								});
				} else {
						// fill with empty
						arr.fill(CellState::Empty);
				};
				// error code print
				// eprintln!("found \'{:?}\' most satisfies {:?}",arr,requirements);
				Ok(())
		}
		/// does one full step in the solveing process (columns and rows)
		/// and outputs debug info if verbose if true
		fn solve1(&mut self,verbose:bool) -> Result<()> {
				// solve columns
				let mut col_num : usize = 0;
				for mut col in self.grid.axis_iter_mut(ndarray::Axis(1)) {
						if verbose { print!("c{}",col_num)} ;
						// @TODO see if can be done without copying to vector
						// eprint!("col {}, to",col);
						let mut col_vec = col.to_vec();
						Self::impl_requierments(&mut col_vec, self.width, self.columns[col_num].clone())?;
						col.assign(&Array1::from_vec(col_vec));
						// eprintln!("{} with req {:#?}",col,self.columns[col_num].clone());
						col_num += 1;
// .with_context(|| format!(	"column {} was not stored contiguasly in memory and can't be used as a slice",col_num))?

				}
				// solve rows
				let mut row_num : usize = 0;
				for mut row in self.grid.axis_iter_mut(ndarray::Axis(0)) {
						if verbose { print!("r{}",row_num)} ;
						let mut row_vec = row.to_vec();
						Self::impl_requierments(&mut row_vec, self.height, self.rows[row_num].clone())?;
						row.assign(&Array1::from_vec(row_vec));
// 						Self::impl_requierments(&mut row.as_slice_mut().with_context(|| format!(
// 																				"row {} was not stored contiguasly in memory and can't be used as a slice",row_num))?
// 																		, self.width, self.columns[row_num].clone())?;
						row_num += 1;
				}
				return Ok(());
		}
		/// solves the board and outputs debug info if verbose if true
		pub fn full_solve(&mut self,verbose:bool) -> Result<()> {
				let mut solve_num = 0;
				while !self.complete() {
						if verbose{ print!("solve # {}:",solve_num) };
						self.solve1(verbose).with_context(||
																							format!("failed at solve # {}, current board:\n{}",solve_num,self.pretty_grid_str()))?;
						if verbose{ print!("\n{}\n",self.pretty_grid_str()) };
						solve_num+=1;
						// return Err(anyhow::anyhow!("exit"));
				};
				Ok(())
		}
		/// prints the grid in an astheticly pleaseing manor
		pub fn pretty_grid_str(&self) -> String {
				let mut out_str : String = "".to_string();
				let mut last_row = 0;
				for ((y, _x), value) in self.grid.indexed_iter() {
						if y != last_row {
								out_str += "\n";
						}
						out_str += format!("{}",value).as_str();
						last_row = y;
				};
				out_str
		}
}

impl fmt::Display for CellState {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
				let c = match self {
						Self::Filled => "█",
						Self::Empty => " ",
						Self::Unknown => "?",
				};
        write!(f, "{}", c)
    }
}

impl BitAnd for CellState {
    type Output = Self;

    // rhs is the "right-hand side" of the expression `a & b`
    fn bitand(self, rhs: Self) -> Self::Output {
				match (self,rhs) {
						(Self::Unknown, b) => Self::Unknown,
						(a, Self::Unknown) => Self::Unknown,
						(Self::Empty,Self::Empty) => Self::Empty,
						(Self::Filled,Self::Filled) => Self::Filled,
						(Self::Empty,Self::Filled) => Self::Unknown,
						(Self::Filled,Self::Empty) => Self::Unknown,
				}
    }
}
#[cfg(test)]
mod test {
		use super::Board;
		use super::CellState;
		const E : CellState = CellState::Empty;
		const U : CellState = CellState::Unknown;
		const F : CellState = CellState::Filled;


		#[test]
    fn test_impl_req1() {
				let mut arr1 = vec![U;5];
				let _ = Board::impl_requierments(&mut arr1,5,vec![5]);
        assert_eq!(arr1, vec![F;5]);

    }
		#[test]
		fn test_impl_req2() {
				let mut arr1 = vec![U;5];
				let _ = Board::impl_requierments(&mut arr1,5,vec![4]);
        assert_eq!(arr1, vec![U,F,F,F,U]);
    }
		#[test]
		fn test_impl_req3() {
				let mut arr1 = vec![U;10];
				let _ = Board::impl_requierments(&mut arr1,10,vec![7]);
        assert_eq!(arr1, vec![U,U,U,F,F,F,F,U,U,U]);
    }
		#[test]
		fn test_impl_req4() {
				// bug found. this board will not advance once it reaches this state.
				//      ███    ███
				// ██   ██  ██████
				// ██   █ ████████
				// ███  ███       
				// ███ ██ ████    
				//   ███  ██ ??█  
				//   ██ ███  ???? 
				//   ███████ ???? 
				//  █████ ████  ██
				// ██ ██   ?███??█
				// █  ██   ?███??█
				//    ███  ?███? █
				//    █████ ███ ██
				//    █████ ███ █ 
				//     ███ ?█?    
				// {
				// "columns": [[4,2],[4,2],[6],[9],[2,8],[5,3,4],[2,1,2,3],[1,7,2],[1,2,3,1],[2,1,7],[2,2,5],[2,1,5],[3,2,2,1],[3,3,2],[3,5]],
				// "rows": [[3,3],[2,2,6],[2,1,8],[3,3],[3,2,4],[3,2,3],[2,3,2],[7,1],[5,4,2],[2,2,4,1],[1,2,4,1],[3,4,1],[5,3,2],[5,3,1],[3,2]]
				// }

				// test row 6
				let mut arr1 = vec![E,E,F,F,F,E,E,F,F,E,U,U,F,E,E];
				let _ = Board::impl_requierments(&mut arr1,15,vec![3,2,3]);
        assert_eq!(arr1, vec![E,E,F,F,F,E,E,F,F,E,F,F,F,E,E]);

				// new board
				//     ███    ███
				//██   ██  ██████
				//██   █ ████████
				//███  ███       
				//███ ██ ████    
				//  ███  ██ ?██? 
				//  ██ ███ ██    
				// ███████ █     
				// █████ ████ ?█ 
				//█  ██   ?█?█??█
				//█  ██   ████  █
				//   ███  ████  █
				//   █████ ███ ██
				//   █████  ███ █
				//    ███ ? ? ?? 

				// Test col 13
				// this turned out to be an error in data entry and stands in monument of that.
				
				// let mut arr1 = vec![F,F,F,E,E,F,E,E,U,U,E,E,E,F,U];
				// let _ = Board::impl_requierments(&mut arr1,15,vec![3,2,2,1]);
        // assert_eq!(arr1, vec![F,F,F,E,E,F,E,E,U,U,E,E,E,F,E]);

    }





}
