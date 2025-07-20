use bevy::prelude::*;
use crate::guild::guild_core::{Guild, GuildMember, GuildManager, GuildResource, GuildFacility};
use crate::guild::mission::{Mission, MissionTracker};
use crate::guild::mission_board::MissionBoard;
use crate::guild::agent_progression::{AgentStats, AgentProgression};
use crate::guild::agent_behavior::{AgentBehavior, AgentBehaviorType};
use crate::guild::agent_equipment::AgentEquipmentManager;
use crate::components::{Player, Position, Name};
use crate::ui::{UIState, UIAction, UIElement, UIBox, UIText, UIList, UIButton, UIPanel};

/// Guild UI state
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GuildUIState {
    Hidden,
    Main,
    Members,
    Missions,
    Facilities,
    Resources,
    AgentConfig,
}

/// Guild UI resource
#[derive(Resource)]
pub struct GuildUI {
    pub state: GuildUIState,
    pub selected_guild: Option<String>,
    pub selected_member: Option<Entity>,
    pub selected_mission: Option<String>,
    pub selected_facility: Option<GuildFacility>,
    pub scroll_offset: usize,
    pub filter: String,
    pub show_completed_missions: bool,
    pub show_failed_missions: bool,
}

impl Default for GuildUI {
    fn default() -> Self {
        GuildUI {
            state: GuildUIState::Hidden,
            selected_guild: None,
            selected_member: None,
            selected_mission: None,
            selected_facility: None,
            scroll_offset: 0,
            filter: String::new(),
            show_completed_missions: false,
            show_failed_missions: false,
        }
    }
}

/// Guild UI action
#[derive(Debug, Clone)]
pub enum GuildUIAction {
    ToggleUI,
    SetState(GuildUIState),
    SelectGuild(String),
    SelectMember(Entity),
    SelectMission(String),
    SelectFacility(GuildFacility),
    AssignMission(Entity, String),
    ConfigureAgent(Entity),
    SetAgentBehavior(Entity, AgentBehaviorType),
    UpgradeAgentStat(Entity, String);
    }
}n())).chai       stem,
    _render_syild_ui     gu
          _system,action guild_ui_             
 system,nput_d_ui_iil         gu(
      ate, stems(Upd.add_sy       I>()
    dUrce::<Guilt_resouapp.ini
         App) {: &mutf, appbuild(&sel{
    fn gin uildUIPlun for Gl Plugi

impldUIPlugin;ruct Gui
pub stuild UI for g Plugin

///
    }
}
        }          }
   }        
       ll_y += 1;  ski             
                               }));
    
          lor: None,          co         e),
     valukill, }: {}", s!("{t: format      tex              
    skill_y,     y:                      x: 47,
                 
     IText {t(U:Texent:emush(UIEls.pntui_eleme              ls {
       &stats.skil inlue)ill, va  for (sk              ;
skill_y = 25t mut        le                 
     
   }));                 None,
    color:           ,
     to_string()s:".: "Skill   text              y: 24,
                       45,
           x:     t {
     ::Text(UITexIElementments.push(U      ui_ele          list
   // Skills        
                      }
                1;
+=    attr_y                
                  
          }      
              }));                rue,
  enabled: t                           ),
 )))tr.clone( aty,entitt(tStaUpgradeAgenAction::ew(GuildUIx::nom(Bostn::Cutio: UIAc     action                      (),
 tringade".to_s"Upgrt: tex                        
     1,height:                       ,
      10h:    widt                 
       : attr_y,        y                  
  : 30, x                       tton {
    on(UIBulement::ButtUIEments.push(  ui_ele                    ts > 0 {
  poinstat_ble_stats.availa    if             
    ablepoints availe button if upgrad   // Add                    
           
           }));             : None,
    color                       value),
tr,  {}", at"{}:xt: format!(       te           y,
      tr_y: at                       
 x: 11,                   {
     Text Text(UInt::UIEleme.push(mentsi_ele        u       
     butes {s.attri) in &statueattr, valfor (               
 25;y = ut attr_     let m      
                    ));
        }        one,
 color: N                   ),
 .to_string(:""Attributestext:                      y: 24,
                 
  : 9, x                   t {
xt(UITex:Teement:ush(UIElts.pui_elemen            
    ibute list  // Attr                  
       ));
      }              ,
  None color:               ts),
    at_poinable_stavailstats.: {}",  Points Stat"Available!(rmat: fo   text        ,
            y: 22                  9,
        x:        {
    IText ::Text(UIElementents.push(U     ui_elem       
               
            }));     one,
     Ncolor:                l),
    levece_to_next_ats.experienstperience, ts.exlevel, stats.stae: {}/{}", xperiencvel: {} | E"Le!( format text:         
           y: 21,           
          x: 9,                 {
 UIText t::Text(sh(UIElemennts.pu   ui_eleme           
                  ;
}))            e,
    : truder       bor            ring()),
 to_sttion".Configuras "StatSome(    title:                
 : 15,eight          h       ,
   idth: 66           w         y: 20,
             
       : 7,           x        {
  Box(UIBox(UIElement::.push_elements   ui       ts {
      tats) = sta Some(s      if letn
      atioigurconf Stats          //      
                }
 = 1;
     +     y           
        
        }));              : true,
   enabled                  ))),
 r_type behavioty,avior(entigentBehon::SetActinew(GuildUIABox:::Custom(tion:: UIAcaction                    _type),
havior beted,?}", selec{}{:mat!("   text: for               ght: 1,
           hei          0,
    width: 3           
       y,             9,
            x:      
         Button {(UIutton:B(UIElement:ents.push ui_elem                         

       };               "[ ] "
                   else {
    }               "[X] "
        
           ior {rent_behavcur == havior_typef beected = i let sel       {
        s  in behaviorr_typeehavio     for b      2;
  = 1ut yet m       l         

            ];  ,
      ectiveType::Protviorha  AgentBe           
   eFocused,rcType::Resouavior   AgentBeh           edy,
  pepe::SntBehaviorTy Age            gh,
   ThorouorType::avi    AgentBeh       ed,
     nce::BalaTyptBehavior        Agen
        tious,rType::CauviotBeha   Agen          essive,
   pe::AggriorTyentBehavAg             = [
    ehaviors     let b   tions
    ior op// Behav           
       ));
         }
         e,r: Noncolo       
         ior),ehavrent_bur {:?}", cehavior:Current Brmat!("text: fo               y: 10,
           
             x: 9,         Text {
::Text(UIElementush(UI.pnts  ui_eleme           
          );
 r_typevio| b.behaed, |blanciorType::BatBehav_or(Agenmapior.ior = behavnt_behav   let curre
              ;
           }))       true,
  r:    borde        
    )),ng(strin".to_gurationfiBehavior Come("e: So        titl,
        t: 10igh  he          : 66,
    th         wid         y: 9,
           : 7,
          x{
         Box(UIBox nt::lemesh(UIEs.pument      ui_ele
      figurationconavior / Beh       /
              
   ));    }
        lor: None,   co   
          ion),ializatpec, member.stion: {}"izat!("Specialma for     text:        7,
             y:7,
                x: 
       xt {ext(UITement::T(UIElets.pushelemen   ui_
                     ;
  }))   
       r: None,colo           str),
     me_", na"Agent: {}xt: format!(   te                y: 6,
          
     x: 7,       t {
       Texxt(UIIElement::Te(Us.push ui_element  
         nfoAgent i       // 
              ());
   n.name.clone, |n| _string().toown"r("Unknap_or = name.m_stame  let n
          entity) {uery.get()) = agent_qracker, nameession, t progrstats,ior, er, behav(_, memb  if let Ok(   mber {
   d_meui.selecte) = guild_ntitylet Some(e
    if ment>,
) {IEleec<U: &mut Vlements>,
    ui_e&Name>) Option<racker>,nTtion<&Missioession>, OptProgron<&Agen, Optis>Stat&Agenton<r>, Optihavio&AgentBeon<ember, Opti, &GuildMuery<(Entity: &Qent_query,
    agildUI&Gu_ui: guild   
 screen(config_t_nder_agenfn ren
tion screenfiguragent cor the a

/// Rende}}
     }
  
      }));          r: None,
    colo             ,
string()able.".to_data availe flow ourc"No res text:            4,
            y: 2     : 9,
           x
        ext(UIText {:TUIElement:push(nts.ui_eleme     ow
        flourceresactual d show n, you woulementatioreal impl// In a             
          }));
           true,
    order:   b          
   ,to_string())urce Flow".e("Reso  title: Som            ght: 10,
     hei             th: 66,
        wid2,
         y: 2             : 7,
        x        UIBox {
  :Box(ent:lemush(UIEelements.p       ui_nses
     /experce income // Resou     
                }
          = 1;
        y +    
                         }));
           
    e, Nonor:ol    c               , ""),
 e(), amountam, resource.n} {:<30}"0} {:<10"{:<2format!( text:            
              y,               x: 9,
             {
      UIText ement::Text(UIElush(s.pi_element     u          
                p_or(&0);
 ).unwrasourceces.get(&resourd.ret = guillet amoun            () {
    source::alln GuildReource i    for res
         mut y = 8;    let       ce rows
 / Resour     /   
             );
        })       r: None,
    colo           ,
 iption")Descr, ", "Amount"urce", "Reso{:<30}":<10} 0} {t!("{:<2 text: forma                   y: 7,
           
 : 9,      x          {
 ext::Text(UITentsh(UIElemts.puemen ui_el       aders
     // He         
                }));
        r: true,
  borde          
      )),ng(rito_stes".urc"Guild Resoome(   title: S            15,
 t:      heigh         : 66,
  dth          wi
      6,      y:              x: 7,
             {
Box Box(UIlement::ush(UIEments.p      ui_ele
      sources list // Re  
         {_id) uildd(gt_guilgenager.ld_ma = guiguild)ome( let S    ifd_id {
    elected_guil = suild_id)let Some(g
    if  {Element>,
)ut Vec<UIents: &m  ui_elem
  ion<String>,_id: &Optcted_guild
    selenager,r: &GuildMad_manage,
    guili: &GuildUI  guild_un(
  ees_scrder_resourcen rens screen
fhe resource/ Render t  }
}

//    }
     }
               }
       ));
          }                 e,
or: Nonol        c          
      t_text, text: cos                   y: 35,
                       ,
     : 9     x               ext {
    ITnt::Text(Upush(UIElemeements.     ui_el                   
         }
                   
        len() - 2);xt.ate(cost_tetext.trunc    cost_                   ) {
 th(", "s_wi_text.endst if co                  space
 nd  comma a trailingRemove      //                  
                        }
             e()));
ame.n, resourc", amount} {}, at!("{sh_str(&formtext.pu    cost_                    {
&cost nt) in ouresource, am      for (            
                      st();
.build_coityst = facil     let co          ();
     ringo_stst: ".t "Cot =st_texco    let mut                 t
 build cos// Show              
                       }));
                       d: true,
nable           e          ))),
   acilityy(facilitBuildFUIAction::w(Guildtom(Box::nection::Custion: UIA       ac          (),
       ringity".to_stBuild Facilext: "     t              1,
      eight:       h                 idth: 20,
     w             34,
      y:                             x: 9,
                    ton {
utton(UIButUIElement::Bnts.push(   ui_eleme                 ld button
  // Bui                  lse {
   } e            }
                   ));
    }                 e,
     : Non     color                     st_text,
  xt: co  te                    
      35,     y:                     x: 9,
                            
   t {Text(UITex::ementElush(UI.pementsi_el      u                  
                   
      }                       );
 - 2len()_text.(costatext.trunc     cost_te                       ") {
, _with("ndscost_text.e    if                e
     and spacomma e trailing c/ Remov  /                      
                           }
                 
    ));rce.name()unt, resoumoupgraded_a", "{} {}, tr(&format!(sh_spucost_text.                        
    er) as u32;ltiplimu32 * level_*amount as famount = (aded_ upgr     let                      ost {
  &base_count) in, amresource    for (            
                             .0;
    0.5 + 1 f32 *evel as instance.lier =tipllevel_mullet                        t();
 d_cos.builtyost = faciliet base_c       l              ing();
   .to_strst: "xt = "Cocost_tet let mu                       e cost
 pgradhow u     // S                    
                       }));
                        
abled: true, en                 
          ity))),il(faclityaciadeF:Upgrn:tio(GuildUIAc:new(Box:on::Customtion: UIActi    ac                      (),
  ".to_stringtyade Facilixt: "Upgr te                     ,
       height: 1                           20,
 dth:          wi                
     y: 34,                       9,
  :          x                {
    IButton::Button(Ush(UIElement_elements.puui                       vel < 5 {
  instance.le        if           on
 de buttgraUp         // 
                              
         }));           None,
      color:                    ss()),
enence.effectiv insta", {:.1}ss:venectirmat!("Effext: fo        te               ,
    y: 32                     9,
  x:                       t {
UITexment::Text(s.push(UIElelement  ui_e                 
                     ;
    }))              None,
      color:           
          )),f.len(aftance.st}", ins {"Staff:!(matfor   text:                     
 : 31,      y                  x: 9,
                       ext {
 :Text(UITUIElement:ents.push(lem  ui_e                        
           );
           })           r: None,
      colo               el),
    e.levstanc: {}", inLevel: format!("ext    t                 30,
    y:                   ,
      x: 9                      
 (UIText {t::Textemens.push(UIEli_element u            ) {
       cilitys.get(&faitieiluild.facnce) = ginstaome( let S  if             uilt
 details if btance cility ins  // Show fa                    
           }));
         
      : None, color            )),
       ion(descriptcility.n: {}", fa"Descriptiomat!( text: for                  29,
  y:           
          x: 9,                   (UIText {
extnt::TlemeUIEh(ents.pus     ui_elem          
                 }));
               one,
 color: N               
     ),e()acility.nam{}", fe: ormat!("Nam   text: f              ,
         y: 28             9,
     x:                 Text {
ent::Text(UI.push(UIElements_elem ui         
                ;
          }))        ue,
    r: trdeor    b          ),
      ()o_string".tetailsFacility Dme("tle: So   ti               ,
  : 10ight        he             66,
     width:        27,
           y:         ,
           x: 7           
      (UIBox {ent::Boxush(UIElemnts.p    ui_eleme       {
     ity ed_facillectild_ui.sety) = gu Some(facili let       if   selected
  ls if cility detai    // Fa 
             
               }      }
             reak;
          b        
  unt >= 15 {     if co        
                  += 1;
  count               y += 1;
                      
             }));
             : true,
 abled en               y))),
    acilitlity(*f:SelectFacidUIAction:ilGum(Box::new(ion::Custoct UIAn:      actio              tus),
aff, sta, st), level.name(facility", <20}{: {:<10} <20} {:<10}"{:t: format!(    tex            ,
     1ht:ig     he         62,
      th: id       w        ,
             y          x: 9,
                   ton {
   on(UIBut:ButtIElement:push(U_elements.ui                  
         ;
                }one
             N          lse {
      } e            0))
55, 255,      Some((2       
        *facility) {ty == Some(cili.selected_fauid_= if guilcolor      let             facility
cted seleight   // Highl                
                };
          )
   t Built" "No0,0,      (             {
    } else               ilt")
"Buf.len(), e.stafl, instanc.leve (instance             {
       ity).get(facililitiesguild.factance) = inse(f let Som = if, status)af, stvelet (le  l            es {
  _facilitiy in visiblefor facilit            
      5);
      ke(1.tastart_idx)kip(er().scilities.itfaties = all__faciliet visible       l    _sub(1));
 saturatingities.len().n(all_facilll_offset.miild_ui.scro guidx =  let start_
          et offscroll // Apply s  
                  
   l();ty::alliFacis = Guild_facilitielet all           
 litiessible faciGet all pos      //    
              nt = 0;
 mut cou  let          
 ;t y = 8      let mu      rows
 ity/ Facil     /         
    
       }));          
  None, color:        ,
       Status"), ""Staff"", , "Levelty"cili<20}", "Fa} {::<1010} {:<20} {:<at!("{: formxt    te            y: 7,
            
         x: 9,           UIText {
:Text(lement:ts.push(UIE   ui_elemen     
    Headers   //              
  }));
           
       rder: true,   bo             
ing()),ies".to_strFacilitGuild e: Some("      titl       t: 20,
   heigh            
    6,h: 6  widt            6,
   y:          7,
          x:             IBox {
x(Uent::Boemsh(UIElpuui_elements.         es list
    Faciliti //         {
  guild_id) d(.get_guild_managerld) = guillet Some(gui     if  {
   _guild_ided) = selectiduild_ome(get Sf l  i) {
  ,
c<UIElement>nts: &mut Veui_elemeing>,
    n<Str_id: &Optioted_guild    selecnager,
GuildMager: &ild_mana
    guldUI,: &Guild_ui
    guireen(acilities_scr_f
fn rendees screenciliti fa/ Render the
//
}
   }
    }    }
                }
  
             }                    }));
                       None,
r:   colo                   (),
      .to_stringn"s missiohiign tr to asslect a membeext: "Se  t                          y: 37,
                           ,
 x: 9                        Text {
    xt(UIt::Te(UIElemenements.push  ui_el                      else {
        }               }));
              
        rue, enabled: t                    ,
       one())))ssion.id.cl_member, mictedion(sele:AssignMissAction:dUIox::new(Guilstom(Bction::Cu action: UIA                         ),
  string(".to_berected Memto Seln igtext: "Ass                        ht: 1,
            heig                 ,
   width: 30                            ,
  y: 37                    
      9,      x:                      utton {
 ton(UIBment::Buts.push(UIEle ui_element               
        er {cted_membuild_ui.seleember) = gted_m Some(selec      if let   
           ilable {s::AvaatusionSt::Misypesmission_tuild:::grate:status == c if mission.         tons
      Action but       //              
           
          }    y += 1;
   d_rewar                           
             }));
                    e,
  color: Non                    ()),
  tioncripes1, reward.d+ , i ("{}. {}"xt: format!          te          rd_y,
     rewa       y:                ,
  42     x:              ext {
     xt(UITnt::Teush(UIEleme.p_elementsui            {
        take(3) umerate().).enards.iter(on.rewn missiward) ii, refor (                33;
 reward_y =mut       let                     
        }));
             None,
        color:          
    _string(),rds:".tot: "Rewa   tex             32,
            y:              x: 40,
                 Text {
  ext(UIIElement::Th(U.pusementsui_el           ards
     / Rew    /                   
         
          };
      y += 1  obj_           
                         }));
               
       None,    color:            ,
         ogress)), prion(scripte_type.deve.objectiv+ 1, objecti", i } - {} {}. format!("{xt:    te          
          : obj_y,        y            
      x: 11,                     t {
 xt(UITex:TeUIElement:s.push( ui_element          
                        };
                  (),
       tringd".to_s Starte"Not    _ =>                   ing(),
  tr".to_sd => "FaileFailed::veStatusionObjectis::Miss_typeission:guild::m     crate:                  
 ing(),".to_strletedd => "CompCompletes::StatuectiveionObjpes::Misson_tyuild::missi    crate::g                       },
                     total)
 ", current,"{}/{} format!(                          > {
 al } =tot { current, :InProgressatus:ObjectiveStssiones::Mi:mission_typild:te::gu   cra                    
 atus {ective.stobj match & progress = let                 ke(3) {
  merate().taer().enues.it.objectivin missionective) bj, ofor (i              3;
   = 3et mut obj_y          l
                    ;
      }))         ,
   r: None        colo       ng(),
     ".to_striives:bject"Otext:             
         y: 32,          ,
            x: 9                 t {
:Text(UITexement:push(UIElts.i_elemen      u        
  esectiv Obj     //        
                       }));
         
   one,: N  color                
  iption),ion.descrmission: {}", "Descriptat!(text: form            
             y: 31,         
        x: 9,         
         xt {ITet(Ulement::Texh(UIEments.pus    ui_ele          
                }));
           ,
        Noner: colo         
          .status),", mission {:?}t!("Status: text: forma                     y: 30,
                 x: 9,
                   {
  IText t(Uexent::Tush(UIElemnts.pi_eleme     u      
                       }));
             None,
 color:                 
    name()),culty.sion.diffi, misiculty: {}"Diff("ext: format! t                 : 29,
       y            x: 9,
                    {
   xt(UIText t::Teemens.push(UIElent_elem   ui             
              ;
        }))        : None,
   color           ,
        n.name){}", missioName: ("t!text: forma                28,
               y:         
  9,      x:          ext {
    (UITnt::TextIElements.push(Ueme_el    ui                 
  ;
           }))             : true,
  border                 ing()),
  s".to_striltaMission Dee(" title: Som             12,
      t:        heigh           dth: 66,
       wi        ,
         y: 27             7,
              x:          x {
  ox(UIBoement::Bs.push(UIElement   ui_el             id) {
ssion__mission(mi_board.getmission(mission) = et Somef l           imission {
 lected_ld_ui.se&guiid) = e(mission_t Som if le      ed
 selecttails if ission de   // M
               }
      }
            break;
                {
 t >= 15coun    if     
               += 1;
 t     coun       += 1;
        y       
           );
     })      d: true,
  enable           )))),
    .clone(.idissionion(mctMiss::SeleldUIAction(Guim(Box::newn::Custo UIActio     action:          ,
 ogress)        pr   ,
         tus)ssion.sta}", mi{:?at!(" form          
          name(),ulty.ion.diffic     miss           e, 
    on.nammissi           
         :<10}", } {:<10:<10} {:<30} {("{rmat!xt: fo     te        
   ight: 1,he              dth: 62,
  wi               
    y,          x: 9,
                   tton {
tton(UIBuElement::Bupush(UIelements.       ui_     
                 };
      
 _string()"".to           lse {
         } e      ))
  centage(ss_per.progremission0}%", "{:.rmat!(        fo    ess {
    :InProgratus:sionStpes::Mismission_tyate::guild::atus == crn.stissiof ms = igresprolet                 
    };
                 None
               else {
 }          255, 0))
  me((255,        So
         id) {(&mission.ef() == Somemission.as_rd_ecte_ui.self guildor = i   let col
         issionected mghlight sel   // Hi      {
    le_missionssibon in vi for missi   
         
   take(15);rt_idx).r().skip(state.isionsered_missions = filtisible_mis let v);
       ub(1)turating_s.len().sassionsfiltered_mit.min(sel_offi.scrol_uuild_idx = gart      let st
  ll offsetpply scro      // A  
       );
    .collect(
             })    
     }            ons,
   ailed_missi_fhowui.sild_red => guExpisionStatus::::Mis_typesonissite::guild::m cra                   |
:Failed tus::MissionStatypes:::mission_guild    crate::               missions,
 pleted__comshowild_ui.ted => guCompletus::ssionStaMin_types::ioguild::misse::       crat     
        > true,gress =tus::InProStaes::Missionsion_typ::guild::mis      crate        |
       AssignedStatus::ones::Missiion_typmissate::guild::cr                  ilable |
  Status::AvaonMissitypes::ssion_::mi::guild   crate        
          {tatus.sch m mat         
      r(|m| {.filte            ter()
into_imissions.d_ilion> = guc<&Mississions: Vetered_met fil      ln status
  s based oter mission/ Fil /       
      ild_id);
  _by_guild(guonsget_missin_board.ns = missiosiold_misguit 
        les guild for thiet missions G      //   
  0;
     mut count =  let  9;
       t y =  let muws
       ro// Mission      
    ;
         }))ne,
       color: No     
     rogress"),", "P "Statusficulty","Dif, ", "Name"0} {:<10}0} {:<1} {:<130at!("{:<text: form        8,
               y:   x: 9,
        Text {
   (UI:Textent:em(UIEllements.pushui_es
        / Header 
        /           }));
ne,
    No   color:         ),
 " } { " "X" } elsens { _missioled_ui.show_fai guild      if         ,
 e { " " }} els"  "Xissions {ed_mw_completld_ui.sho     if gui           " " },
" } else { { "Xns siomisfailed_ow_d_ui.sh && !guil_missionsetedmpl_cold_ui.showui!g      if          ",
 ed [{}] Failpletedble [{}] Comvaila [{}] Aow:at!("Shform   text:          : 7,
  y    : 9,
                 x(UIText {
 nt::Texth(UIElemeusments.p_ele        uins
er optio    // Filt
            ));
     },
    trueder:    bor     ),
   g()".to_strinMissionsvailable ("A Some   title:          20,
ight:          he: 66,
  idth     w            y: 6,
,
          x: 7
         {x(UIBox nt::Boh(UIElemements.pus     ui_elet
   sion lis // Mis       
ld_id {elected_guiid) = sild_ome(gulet S{
    if nt>,
) UIEleme &mut Vec<ents:_elem
    uion<String>,ld_id: &Optied_gui selectard,
   ssionBo&Miard: bo    mission_I,
i: &GuildU_u guild   screen(
r_missions_
fn rendes screenssionender the mi

/// R}
    }
}       }
      
       }               }));
             
        led: true, enab                   ,
    entity)))nt(selected_ConfigureAgeAction::dUIw(Guilx::ne::Custom(Botiontion: UIAc   ac                    ng(),
 o_stri".t Agenture"Config: text               
         1,ght:        hei            0,
     idth: 2           w          ,
        y: 35                  2,
      x: 3               tton {
    on(UIBuuttnt::Bpush(UIElemeements._el     ui                
            ));
                  }
         rue,led: t   enab                )),
     ions)e::MissdUIStattate(GuiltS::SeUIAction(Guildnewm(Box:::CustoUIAction:on:        acti               ,
  ()stringission".to_"Assign M    text:                     1,
 eight: h                    h: 20,
         widt           ,
       : 35     y                : 9,
              x           {
   (UIButtonuttonement::Bush(UIElts.p ui_elemen                  ttons
 // Action bu                           
              }
                  ));
        }         
        None,or:     col                      
  s.len()),ssioned_mier.completckrans: {}", td missioomplete"Cformat!(ext:    t                     3,
        y: 3                      40,
  x:                        
     ext {nt::Text(UITemeh(UIElents.pus   ui_elem              
                        ;
        }))                    None,
         color:                   ,
    us_statext: mission  t                         : 32,
         y                0,
           x: 4              {
       UIText xt(ment::TeEles.push(UIement   ui_el                       
             ;
              }                
   to_string()".Available   "                      
    {      } else                
   mission_id)on: {}",!("On missi    format                 n {
       ssio.active_mitracker= &id) ission_ Some(m = if let_statust mission    le                   er {
 ck = traer)ack(trme   if let So                
                      }
                   );
  })                 None,
          color:                    el),
    to_next_leve_xperiencts.eience, stats.exper/{}", starience: {}!("Expemator text: f                     
      ,y: 31                   
             x: 40,                      t {
  xt(UIText::TeUIElements.push(en     ui_elem           
                          ));
           }                one,
   or: N   col                  
       .level), {}", stats"Level:t!(ext: forma        t                   : 30,
      y             
           40,          x:          {
        UIText t(::Texments.push(UIEleui_element                       stats {
 ) = tatset Some(s       if l                
                }
                  ));
              }          None,
     color:                         
  or_type),vior.behavi", beha: {:?}"Behaviorformat!(    text:                       : 33,
     y                     9,
       x:                     t {
     UIText(:TexElement:ts.push(UIi_elemen    u               
      behavior {ior) =avome(beh if let S                  
                   }));
                    None,
       color:                    ,
ation)er.specializ}", memb {ization:ial"Spec(t: format!        tex         ,
          y: 32                : 9,
        x                   
  IText {xt(U:Tement:ush(UIElets.plemen    ui_e              
                 
         }));               None,
  or:    col                e()),
    r.rank.nam{}", membe"Rank: at!(ormtext: f                       
       y: 31,                 9,
          x:         t {
       UITex:Text(nt:h(UIElemets.pus  ui_elemen                 
                  
        }));            None,
         color:                   r),
_st, name"Name: {}"format!(: ext         t             30,
      y:            
              x: 9,                {
   t UITexext(lement::TIEs.push(U_element      ui                    
    
          one());me.cl(), |n| n.nao_string".town_or("Unknapame.m = nt name_str       le        
                       ));
           }         : true,
        border                  ing()),
o_strs".ter Detail"Membitle: Some(       t                 eight: 10,
          h       
       6,dth: 6wi                   : 29,
             y                     x: 7,
               ox {
    ent::Box(UIBElemUInts.push(  ui_eleme                  ntity) {
elected_eery.get(st_qu)) = agenr, nameon, tracke, progressitsavior, staer, beh membk((_,t Of le          i
      er {mbcted_meuild_ui.sele gntity) =d_ecte Some(sele let        if    ed
ctif seletails r de    // Membe          
    }
                   }
              break;
                   = 15 {
  t >coun if                
               1;
    count +=              y += 1;
          
                  );
             })      ,
 abled: true       en           
  ity))),ber(*entn::SelectMemuildUIActioBox::new(Gstom(ion::Cution: UIAct        ac            tus),
, stassionse(), mi.rank.nam member name_str,:<10}",10} {:<10} {{:<20} {:<format!("text:                     : 1,
ghthei                 : 62,
       width               y,
                 9,
      x:             
      n {UIButtoButton((UIElement::s.pushui_element          
                    };
              
    one          N           {
else          } 
      55, 0))255, 2     Some((           ) {
    me(*entityber == So_mem.selectedf guild_uior = iet col      l         d member
 ecteght sel/ Highli /                       
             };
           "
le  "Availab               e {
        } els         
   Mission"      "On        {
      .is_some() clone())on.ssiactive_mi_then(|t| t. tracker.andif = tatuslet s                
ions.len());ssed_mietcompl, |t| t.er.map_or(0racksions = tis let m        ;
       me.clone()).nag(), |n| nrin".to_stknownUn("or= name.map_et name_str     l          s {
  ble_memberker) in visiacme, trber, nay, memr (entit       fo  
             5);
  ).take(1dxkip(start_iter().ss.imberembers = meible_m  let vis         (1));
 ating_subaturers.len().smemboffset.min(ui.scroll_ild_= gu_idx   let start        offset
  ll / Apply scro           /
                  }
   }
               );
     e, tracker)er, nam, membush((entity  members.p           d {
       ld_iuiild_id == *gember.guif m              () {
  .itert_queryn agenker, name) i_, tracr, _, _, tity, membe for (en      
     );:new(ec:= Ver>)> issionTrackon<&M>, Optiion<&Namer, OptildMembety, &GuVec<(Entimbers: let mut me          ild
  this gus for llect member/ Co           /
            
  0;unt =coet mut     l       
 1; y = 1ut  let m       
   r rowsembe  // M            
            }));
   ne,
          color: No           ,
  ")tus"Sta", "Missions "Rank", me",}", "Na10<10} {:<{:<20} {:<10} "{:t!(xt: forma          te 10,
         y:              x: 9,
         
      t(UIText {t::Texemenpush(UIElelements. ui_    s
       eader        // H     
         }));
            
  r: None,      colo
          ilter),_ui.f}", guildFilter: {!("format     text:       y: 9,
                      x: 9,
           {
    ext(UIText lement::Th(UIE.pustsen ui_elem         r
  // Filte         
               
); })         : true,
        border         ring()),
 s".to_stembere("Momle: S  tit         20,
         height:             6,
 width: 6           8,
          y:     ,
      : 7      x         Box {
 x(UIElement::Boh(UIments.pusui_ele             list
  // Members      
             }));
         ne,
      lor: No  co            e),
  ld.nam, gui{}"("Guild: ormat!: ftext           : 6,
             y       x: 7,
             
     {ext(UITextlement::Ts.push(UIEentelem       ui_ame
     // Guild n          {
   guild_id)et_guild(er.gild_managd) = guguilSome( if let        {
 d_guild_id= selecteild_id) e(gu let Som
    if
) {Element>,c<UIt Ve&muments:   ui_ele>,
  >)on<&Nameer>, OptiracksionT Option<&Misrogression>,AgentP Option<&s>,&AgentStat>, Option<viorn<&AgentBehaOptioildMember, y, &Guuery<(Entit&Q_query: gent a
   g>,ion<Strinid: &Optd_guild_  selecteanager,
  : &GuildManagerd_mguilildUI,
    Gui: &d_uuil
    gers_screen(embfn render_mrs screen
behe memnder t

/// Re}
    }
}      = 1;
         y +     ));
       }    e,
 led: truab      en         one()))),
 d(id.cl:SelectGuilldUIAction:Box::new(Guitom(:Cusction:tion: UIA     ac        
   ),vele, guild.leguild.namvel {})",  (Leformat!("{}     text:            ,
ht: 1      heig           62,
th:       wid,
                  y      x: 9,
                 Button {
n(UI:Butto(UIElement:nts.push_eleme     ui
        {.guildsuild_manager &gild) in gu(id,  for = 9;
      ut y  let m         
));
            }
  true,er:    bord       ng()),
  ".to_striildsailable Gu("Avome   title: S         eight: 10,
    h     
   66,   width: 
          y: 8,           x: 7,
        Box {
    t::Box(UIush(UIElemenelements.p
        ui_able guildsavail    // List     
    
           }));ne,
 : Noolor          cring(),
  ".to_sty guild.r of anmembee not a "You art:   tex     
      6,   y:          7,
x:           UIText {
 t::Text(h(UIElemenuslements.p      ui_eelse {
   } }
   );
        })           or: None,
   col            ng(),
  o_strid.".tselecteNo guild : "xt          te      : 6,
       y       
  : 7,        x     
   t {Tex::Text(UI(UIElementlements.push_e ui         
  } else {     }));
              d: true,
 enable            s))),
    ilitieState::FacuildUI:SetState(GUIAction:x::new(Guildn::Custom(Bon: UIActioioact           (),
     _stringtoes".tiliFaciw : "Vie   text           
  t: 1,eigh           h 20,
       width:           32,
             y:  ,
     53        x:      {
    tonUIButButton(ment::(UIEle.pushui_elements           
            
 );       })
     rue, t    enabled:        ns))),
    :Missiotate:ldUIStState(Gui:Sen:UIActio(Guildnewtom(Box::us:CIAction:   action: U         
    ring(),ons".to_stsiis"View M text:            ,
    ight: 1   he            : 20,
  width                32,
        y:    x: 30,
                   IButton {
 ::Button(Untlemepush(UIElements.  ui_e          
           }));
             true,
ed:   enabl           ))),
   e::MembersatUISttState(GuildIAction::Sew(GuildUtom(Box::neon::CusUIActi    action:             
_string(),ers".tomb"View Mext:    te            eight: 1,
    h         20,
    dth:     wi            32,
         y:       
  7,x:                {
 tonUIButton(utent::BIElemh(Uents.pus    ui_elem     s
    button Navigation//               
         }));
        ne,
     Noor:         col       (),
ing".to_stro display. tt activity"No recen:     text            26,
   y:      
            x: 9,            xt {
xt(UITent::TeIEleme(Uts.pushemen       ui_elity
     nt activtual rece acld showu wou, yoentationemeal impl In a r //               
        }));
            rue,
r: t   borde          
   ing()),ty".to_strnt ActiviSome("Rece   title:            ht: 7,
         heig   66,
      idth:       w     
      y: 24,            x: 7,
                IBox {
   (UBoxnt::Eleme.push(UItsmen  ui_ele
          ent activity // Rec         
      }
                    += 1;
           y    );
         })      None,
      color:              ),
    stance.level.name(), inityfacill {})", Leveat!("{} (formtext:            
           y,         ,
         42 x:                   Text {
 t::Text(UIh(UIElemenlements.pus    ui_e           
 facilities {n nce) istacility, infa   for (  ;
       take(5)iter().cilities.= guild.fafacilities    let     17;
      y =             
         ;
      }))     : true,
   der      bor         
 ()),".to_string"Facilitiesome( S  title:      7,
        ht:       heig         33,
      width: 
            16,  y:         0,
      4      x:    
      {ox t::Box(UIBIElemenments.push(Ui_ele          uummary
   slities    // Faci            
      
      }    
    ;= 1        y +         }));
         ne,
      color: No                  
  nt),), amouce.name( resour}: {}",rmat!("{fotext:                      y,
                : 9,
            x       Text {
    ::Text(UIIElementpush(Unts. ui_eleme         0);
      .unwrap_or(&ce)ouret(&resresources.guild. = gamount      let 
          ) {:all(ldResource:Guie in esourcr r          fo;
  17t mut y =           le
            }));
         true,
     : rder       bo        ,
 string())".to_esurce("Resoomtitle: S               ht: 7,
      heig
           width: 30,            ,
           y: 16       x: 7,
          
        IBox {nt::Box(UUIElements.push(_eleme         uiy
   marsumources es    // R
               ));
       }
          e,oncolor: N             
   .clone(),scriptionxt: guild.de     te
           12,    y:             x: 9,
            {
    xt(UIText ment::TeUIEles.push(element ui_        
             );
       })   e,
    ru: t     border
           g()),rintion".to_stDescrip Some("      title:   
       t: 4,      heigh          dth: 66,
   wi          1,
    y: 1               ,
        x: 7        Box {
Box(UI(UIElement::ements.push      ui_eln
       Descriptio   //        
            
     }));   ,
     or: None col         ()),
      bers.lenild.mem, gu {}"embers:format!("M:        text,
              y: 9           ,
 x: 7         
      UIText {ment::Text(les.push(UIE_element    ui          
      ));
         }ne,
       No color:         ,
       utation).rep", guild{}ation: ("Reputmat!t: for       tex,
                 y: 8    7,
          x:    {
        xtText(UITeIElement::push(Uents.emel    ui_        
    ;
             })),
       olor: None       c     el),
    uild.lev{}", gvel: t!("Lext: forma    te       
     7,        y: 
        ,     x: 7           t(UIText {
ent::Texsh(UIElem.punts    ui_eleme       
    
         }));       None,
     r:   colo          e),
     guild.nam}",: {"Guildt: format!( tex              
  6,        y:           x: 7,
             xt {
xt(UITeent::Tesh(UIElem_elements.pu  ui      nfo
    / Guild i   /
         id) {d(guild_get_guilanager.) = guild_mldet Some(gui       if l{
 ed_guild_id ct = seled)(guild_i Some   if lett>,
) {
 enIElemmut Vec<Uts: &i_elemenng>,
    uStriid: &Option<d_il_gued
    selecter,agldManr: &Guild_manage   guildUI,
 i: &Guiuild_u g
   n(ain_screeender_mn rld screen
fmain guie r thende
/// R);
}
e,
    })Non   color: "),
     [↑/↓] Scroll| [ESC] Close ormat!("   text: f: 41,
             y  x: 7,
{
      xt ext(UITet::Tush(UIElemenents.pi_elem   u
 rote 
    // Fo}
   
     _ => {}   ),
     ui_elements_query, &mutui, &agentild__screen(&gut_configder_ageng => renfientCon::AgStatedUI     Guil,
   ements)&mut ui_eluild_id, cted_gr, &seleild_manageui, &guguild_es_screen(&resourc> render_urces =esoldUIState::R   Gui     ,
nts)i_elemed_id, &mut ucted_guilger, &selemanaui, &guild_een(&guild_ities_screr_faciles => rendte::Faciliti  GuildUISta     ments),
 mut ui_eleld_id, &cted_guiboard, &selemission_ui, &reen(&guild_ssions_scmi render_ =>ionste::MissildUISta        Guments),
_eley, &mut ui_querid, &agentuild_&selected_ger, nag_ma &guild(&guild_ui,creenmbers_sme> render_Members =::ateldUISt    Gui   ts),
 ut ui_elemenild_id, &mguselected_ager, &d_manil, &guild_uigun_screen(& render_maiMain =>ldUIState::      Gui{
  te ui.stamatch guild_     on state
ontent basedriate cpropRender ap
    // 
    
    }));: None,color       "),
 urces] Resoies | [5] Facilits | [4ion Missbers | [3]] Mem| [2in "[1] Ma(mat!xt: for     te
   4,y:        x: 7,
      
   UIText {t(ent::Tex(UIElemlements.pushbs
    ui_eion ta/ Navigat    
    /    }));

true,order:  b
       to_string(),ement".Manag"Guild     title: t: 40,
    gh        hei0,
   width: 7
     : 2,    y: 5,
    
        x{(UIPanel nt::PanelIElements.push(Ume  ui_ele  
n container/ Mai
    
    /));ne(ild_id.cloayer_gur(plne().old.clo_guiselected guild_ui._guild_id =edselect let 
   ild guedelect Get s
    //    ;
   }None
         } else {
 
          }None
             {
 else      }
   .clone())r.guild_iduild_membe(g        Some   ntity) {
 get(player_ery.agent_que _)) = , _, _, _,ember, _guild_m, t Ok((_  if le{
      next() ery.iter()._qu playery) =titplayer_ene(Som let _id = if_guildlet player guild
    Get player's  //    
   }
 return;
   {
        en e::HidduildUIStat== Gtate ld_ui.sif guisible
    is viender if UI y r // Onl
   );
    ar(ts.clelemen
    ui_eelementsI  Ungr existi    // Clea>>,
) {
ElementsMut<Vec<UI: Rentsmemut ui_ele
    ,r>>th<Playeity, Wi Query<Enty:layer_quer    p>)>,
ametion<&N>, OpsionTrackerMisption<&ession>, OogrPrtion<&Agent, Ops>AgentStat, Option<&entBehavior>tion<&Agmber, OpGuildMeEntity, &ery<(query: Qu
    agent_d>,issionBoarard: Res<Mbo    mission_anager>,
GuildMer: Res<guild_manag>,
    s<GuildUIguild_ui: Re  ystem(
  _sderi_ren_upub fn guild UI
e guildring th for rende/ System

//    }
}    }

             }         }
  
        },                s;
    missioniled_i.show_fa!guild_ussions = failed_mi_ui.show_ld        gui                 {
=>Missions eShowFailedtion::TogglAc     GuildUI         ,
                 }        
 missions;ompleted_ld_ui.show_cons = !guisieted_mismpl_cohowild_ui.s gu                 {
      ons => dMissitempleShowColeon::TogguildUIActi           G        
     },             e();
   lter.clonr = fi_ui.filte       guild                  => {
r(filter)on::SetFilteldUIActi  Gui       
           },                    += 1;
et offsui.scroll_ld_      gui                  wn => {
:ScrollDotion:Ac GuildUI            ,
              }          }
                         -= 1;
  set roll_off_ui.scild     gu                {
       set > 0 ll_offd_ui.scroil  if gu                    {
  llUp => crotion::S GuildUIAc           
             },           
      }                   
        }                    ;
   facility)ility(*d_facld.buil         gui                      id) {
 mut(guild__guild_ager.getanuild_md) = get Some(guil      if l                      guild {
i.selected_uild_u) = &ge(guild_id Somf let      i               acility
    f Build          //           y) => {
   ity(facilitFacilild::BuuildUIAction           G     
     },          
           }                   
        }                    
   *facility);de_facility(d.upgrauil       g                   ) {
      _idguildmut(et_guild_ld_manager.g = guiild)(guSomelet    if                        _guild {
  ctedsele= &guild_ui.d) e(guild_it Som  if le                     
  facility Upgrade       //          
       lity) => {lity(facici:UpgradeFaldUIAction:         Gui                  },
              }
                    
   );, 1tat_name_attribute(sincrease      stats.                      
(*entity) {ry.get_mutque_)) = agent_tats, t s mu((_, _,if let Ok                     nt stat
   grade age   // Up                  => {
    ame)tity, stat_nntStat(enAge:Upgraden:tioildUIAc   Gu      
            },                         }
          ;
        or_type)avi*behior::new(tBehavvior = Agen    *beha                       ty) {
 t_mut(*entiy.geuert_q= agen)) r, _, _ut behavio((_, m  if let Ok                      t behavior
 Update agen     //             
      {e) => vior_typity, behaior(enttBehavetAgen:SdUIAction:Guil           ,
                }          
   g;gentConfidUIState::Aate = Guilstld_ui.         gui          
     ity);ent Some(*ber =cted_memld_ui.seleui g                    => {
   ty) gent(enti:ConfigureAUIAction:ld   Gui                ,
           }          }
                      }
                             entation
 eal impleme used in rld btime wount / Curre_id, 0.0); /missiont_mission(tarr.s tracke                         y) {
      (*entit_mutry.getgent_que)) = aker)(mut trac, _, _, Somelet Ok((_f      i                  er
     ion trackte misseate or upda      // Cr                     
                            ty);
 entiign_agent(*.ass     mission                   
    sion_id) {_mut(misissionoard.get_mssion_bion) = miet Some(miss    if l                   
 on to agentssign missi // A                       > {
) =ssion_idmity, ssion(entin::AssignMitioldUIAc    Gui               },
                ty);
     cili*fa= Some(ed_facility cteleild_ui.s   gu               > {
      lity) =(facicilitySelectFaion:: GuildUIAct        
               },           ;
     one())sion_id.cl(misSomeon = missid_teelec guild_ui.s                      > {
 d) =ion_issmictMission(ction::Sele    GuildUIA        
             },             entity);
   Some(*_member =selecteduild_ui.   g                   y) => {
  mber(entitlectMection::SedUIAGuil                          },
          
    lity = None;cied_faui.select      guild_            ne;
       = Noted_missionselecd_ui.     guil                 e;
   = Non_memberedui.select   guild_                     );
e()ld_id.clonme(gui Soted_guild =elecui.suild_           g        
      => {ld(guild_id)SelectGuin::tio GuildUIAc            },
                  }
                              e;
   truve = ate.acti      ui_st            
          else { }                
        alse;ve = fe.actistat    ui_                     {
    te::HiddendUIStaGuilf *state ==  i                  );
     one(.clate = stateild_ui.st     gu                    {
 =>(state)::SetStateUIActionGuild                      },
                    }
                   e;
   ls faate.active =i_st u                        
   ate::Hidden;uildUISt= Gtate uild_ui.s           g               {
       } else                    true;
  =.active   ui_state                       in;
  ate::Ma= GuildUIStui.state guild_                            en {
ddState::Hi= GuildUI_ui.state =ldf gui   i                      {
=>n::ToggleUI ldUIActio   Gui             on {
    ld_actich gui       mat         >() {
ildUIAction:<Guef:owncast_ron.dstom_acti = cud_action)Some(guilif let             {
) = action ction(custom_aon::Customlet UIActi    if   er() {
  ons.itui_action in r acti   fo>,
) {
 th<Player>y<Entity, Wierquery: Qu
    player_racker>)>,ionTmut Misstion<&ts, OpSta, &mut AgentgentBehaviorty, &mut Ay<(EntiQuery: t_quer mut agenoard>,
   ssionBResMut<Miboard: mission_
    mut dManager>,ut<Guilger: ResMld_mana gui mut>,
   er<UIActionentReadEvi_actions:  ue>,
    mutMut<UIStatate: Resstui_    mut ildUI>,
sMut<Guui: Reuild_  mut gm(
  tection_sysguild_ui_aub fn actions
pUI dling guild  han/ System for

//    }
}n))));
State::Hiddee(GuildUI::SetStatondUIActiuilnew(G::BoxCustom(d(UIAction::_actions.sen     ui   
ape) {eyCode::Escst_pressed(Kut.juinpyboard_f kescape
    i UI with Ese// Clo       
 

    }n)));crollDowdUIAction::Sx::new(GuilBom(::Custod(UIAction_actions.sen        uiDown) {
yCode::d(Kepressest_t.juard_inpueybo
    if k }));
   llUp)tion::ScroldUIAcw(Gui:ne:Custom(Box:ion:UIActnd(_actions.se
        uip) {(KeyCode::Upressedput.just_eyboard_in kling
    if   // Scrol   
     }
 s))));
ceate::Resourte(GuildUISton::SetStaldUIActi(Gui::new:Custom(Boxtion:Ac.send(UI  ui_actions {
      umpad5)yCode::NKeed(esst_prut.juseyboard_inp k::Key5) ||(KeyCodest_pressedard_input.juif keybo }
    )));
   lities)ciFa::IStateuildU(GtState:SeAction:dUIew(Guilustom(Box::non::CtiUIAcnd(ctions.se  ui_a{
      Numpad4) ode::ed(KeyC.just_pressrd_input|| keyboa) :Key4e:KeyCoded(t_press_input.jusf keyboard }
    i;
   Missions))))::ldUIStateui::SetState(GIAction(GuildUnew::om(BoxCuston::end(UIActiui_actions.s{
        mpad3) :Nud(KeyCode:_presse.justinputd_ keyboar:Key3) ||ode:ressed(KeyCjust_pt.eyboard_inpu  if k  }
  )));
  ers)State::MembildUI:SetState(Guon:(GuildUIActiox::newCustom(BIAction::.send(Uctionsui_a       d2) {
 ode::Numparessed(KeyCut.just_poard_inpkeybe::Key2) || KeyCod_pressed(nput.justf keyboard_i
    i }   ))));
ate::MainldUIStui(Gteon::SetStauildUIActiox::new(Gom(Btion::CustIAcions.send(U     ui_act {
   1)NumpadCode::(Keysedpresput.just_yboard_in:Key1) || keCode:sed(Key.just_presyboard_input ke iftes
   staeen UI igation betw
    // Nav    }
    return;

        te::Hidden {ldUISta == Guistatei.guild_u
    if sibleif UI is vither inputs rocess o// Only p    
    ;
    }
oggleUI)))UIAction::T:new(Guildox:stom(B::Cuion.send(UIActactions ui_       de::G) {
essed(KeyCout.just_pr_inpeyboard   if k' key
 UI with 'Gild  gu Toggle //,
) {
   on>ActientWriter<UI_actions: Ev
    mut ui>,e>ut<KeyCodt: Res<Inprd_inpu  keyboaUI>,
  uildui: ResMut<Gmut guild_
    tate>,ISt<UsMuRe_state: ui    mut m(
_systeld_ui_inputub fn guiUI input
pild ling gutem for hand// Sys
/
ons,
}dMissigleShowFaileog,
    TnsissioompletedMoggleShowC Tring),
   r(St    SetFiltellDown,

    ScroollUp,cr),
    SldFacilityy(GuicilitFauild,
    By)Facilit(GuildityacildeF,
    Upgra