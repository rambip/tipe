use yew::prelude::*;
use web_sys::{window, HtmlInputElement};
use itertools::Itertools;
use rand::Rng;

const L:i32 =20;

struct Model {
    n: usize,
    ruban: Vec<bool>,
    solution: Option<(f32, Vec<f32>)>,
}

enum Msg {
     Compute,
     ChangeNPoint(i32),
     ChangeRuban,
}

fn score_repartition_optimale(l: &[bool], n: usize, i_debut: usize, i_fin: usize, memo: &mut Vec<Vec<Vec<(f32, usize, usize)>>>) -> f32 {
    if memo[n][i_debut][i_fin].0 != 0.0 {
        // partie "memoization"
        return memo[n][i_debut][i_fin].0;
    }

    let taille_segment = i_fin - i_debut;

    // est-ce possible de répartir uniformément les n points
    // entre i_debut et i_fin ?
    let equi_repartition_possible: bool = (1..=n-2).into_iter().all(
        |point| {
            let case_ruban = i_debut + point*taille_segment / (n-1);
            if point*taille_segment % (n-1)==0 {  l[case_ruban] || l[case_ruban-1] }
            else { l[case_ruban] }
        }
    );
    if equi_repartition_possible {
        let score = (((n-1) as f32)*((i_fin - i_debut) as f32)).sqrt();
        memo[n][i_debut][i_fin].0 = score;
        score
    }
    else {
        let frontieres = (i_debut+1..i_fin-1)
            .filter(|&i| l[i-1] != l[i]);
        let meilleur_choix = (1..=n-2).cartesian_product(frontieres)
            .map(|(n_points_gauche, i_milieu)| 
                 ( score_repartition_optimale(l, n_points_gauche+1, i_debut, i_milieu, memo) +
                 score_repartition_optimale(l, n-n_points_gauche, i_milieu, i_fin, memo),
                 n_points_gauche, i_milieu
                 )
             )
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or((0., 0, 0));

        memo[n][i_debut][i_fin] = meilleur_choix;
        meilleur_choix.0
    }
}

fn construire_repartition_optimale(n: usize, i_debut: usize, i_fin: usize, memo: &Vec<Vec<Vec<(f32, usize, usize)>>>, points: &mut Vec<f32>) {
    let (_, k, i_milieu) = memo[n][i_debut][i_fin];
    if k == 0 { 
        let taille_segment = (i_fin - i_debut) as f32;
        for point in 1..=n-2 {
            points.push((i_debut as f32)+(point as f32)*taille_segment/((n-1) as f32))
        }
    }
    else {
        construire_repartition_optimale(k+1, i_debut, i_milieu, memo, points);
        points.push(i_milieu as f32);
        construire_repartition_optimale(n-k, i_milieu, i_fin, memo, points);
    }
}

fn repartition_optimale(l: &[bool], n: usize) -> (f32, Vec<f32>) {
    let m = l.len();
    let mut memo = vec![vec![vec![(0., 0, 0); m+1 as usize]; m+1]; n+1];
    let opt = score_repartition_optimale(l, n, 0, m, &mut memo);
    let mut points = Vec::with_capacity(n);
    construire_repartition_optimale(n, 0, m, &memo,  &mut points);
    points.push(0.);
    points.push(m as f32);
    (opt, points)
}

impl Model {
    fn afficher_segment(&self) -> Html {
        let mut shapes = Vec::new();
        for (i, &b) in self.ruban.iter().enumerate() {
            shapes.push(
                html!{
                    <rect
                        x={(50+L*(i as i32)).to_string()}
                        y=50
                        width={(L-2).to_string()}
                        height={L.to_string()}
                        fill = {if b {"grey"} else {"black"}}
                        />
                }
            )
        }
        if let Some((_, sol)) = &self.solution {
            for &f in sol.iter() {
                shapes.push (
                    html!{
                        <circle
                            cx={(50. + f*(L as f32)).to_string()}
                            cy={(50+L/2).to_string()}
                            r=5
                            fill="red"
                            />
                    }
                )
            }
        }
        shapes.into_iter().collect::<Html>()
    }
}

fn ruban_aleatoire(taille: usize) -> Vec<bool>{
    (0..15)
        .map(|i| rand::thread_rng().gen())
        .collect()
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        wasm_logger::init(wasm_logger::Config::default());
        Model {
            n : 40,
            ruban : ruban_aleatoire(15),
            solution: None
        }
    }


    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Compute => {
                self.solution = Some(repartition_optimale(&self.ruban, self.n))
            }
            Msg::ChangeNPoint(a) => {
                self.n = a as usize;
                self.solution = None;
            }
            Msg::ChangeRuban => {
                self.ruban = ruban_aleatoire(15);
                self.solution = None;
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link().clone();
        let slider_value = |e: InputEvent|
            e.target_unchecked_into::<HtmlInputElement>()
            .value_as_number() as i32;

        html!{
            <div>
                <div>
                    <svg height=100 width=1000>
                        {self.afficher_segment()}
                    </svg>
                </div>
                <div><button onclick={link.callback(|_| Msg::ChangeRuban)}>{"Changer le ruban"}</button></div>
                <b> {format!("nombre de points: {}", self.n)} </b>
                <input oninput={link.callback(move |e| Msg::ChangeNPoint(slider_value(e)))} type="range" min="0" step="1" max="100" value={self.n.to_string()} />
                    <div> <b> {format!("score pour une répartition uniforme: {}", ((self.ruban.len() as f32)*((self.n-1) as f32)).sqrt())}</b> </div>
                <div><button onclick={link.callback(|_| Msg::Compute)}>{"Calculer la solution optimale"}</button></div>
                if let Some((x, _))=self.solution {
                    <div> <b> {format!("score maximal: {}", x)}</b> </div>
                    <div>
                    </div>
                }
            </div>
        }
    }

}
fn main() {
    yew::start_app::<Model>();
}
