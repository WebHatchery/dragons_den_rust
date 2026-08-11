use super::*;

impl GameplayState {
    /// Live expedition find-chance. A live Hoard Rush adds a flat "lucky
    /// window" bonus on top of the economy base, then re-clamps to the same 0.95
    /// ceiling — so exploring *during* a rush (which a miss or find ignites) is
    /// luckier, closing the explore→rush→explore loop. The bonus lives here, not
    /// in `economy`, so the balance sim stays an honest core-loop measure.
    pub fn discovery_chance(&self, data: &GameData) -> f64 {
        let base = economy::discovery_chance(&data.config, &self.rates(data), &self.percents(data));
        if self.hoard_rush_active() {
            (base + data.config.hoard_rush_discovery_bonus).clamp(0.0, 0.95)
        } else {
            base
        }
    }

    /// Current expedition cost, rising with treasures already discovered (#6).
    pub fn explore_cost(&self, data: &GameData) -> f64 {
        economy::explore_cost(&data.config, self.persistent.discovered_treasures.len())
    }

    // --- actions ---------------------------------------------------------

    pub(super) fn earn(&mut self, amount: f64) {
        self.run.gold += amount;
        self.persistent.stats.gold_total_earned += amount;
    }

    /// Passive income for one frame; also advances the run clock. Earnings use
    /// the current Hoard Rush factor, so the surge is drained *after* it pays out.
    pub fn tick(&mut self, data: &GameData, dt: f32) {
        self.earn(self.gold_per_second(data) * f64::from(dt));
        self.run.run_seconds += f64::from(dt);
        self.hoard_rush_secs = (self.hoard_rush_secs - f64::from(dt)).max(0.0);
    }

    /// Counts up toward the player's autosave interval (seconds, from settings);
    /// true means "save now".
    pub fn autosave_due(&mut self, interval_secs: f32, dt: f32) -> bool {
        self.autosave_accum += dt;
        if self.autosave_accum >= interval_secs {
            self.autosave_accum = 0.0;
            true
        } else {
            false
        }
    }

    /// Click the hoard; returns the gold gained for "+N" feedback.
    pub fn click(&mut self, data: &GameData) -> f64 {
        let gained = self.gold_per_click(data);
        self.earn(gained);
        self.persistent.stats.clicks_total += 1.0;
        gained
    }

    /// Effective base goblin hire cost after the Goblin Recruiters discount.
    pub fn hire_base_cost(&self, data: &GameData) -> f64 {
        economy::hire_base_cost(&data.config, &self.rates(data))
    }

    /// Hires up to `requested` goblins, buying as many as gold allows.
    /// Returns `(hired, total_cost)`. `requested == u32::MAX` means "buy max".
    pub fn try_hire_bulk(
        &mut self,
        data: &GameData,
        requested: u32,
    ) -> Result<(u32, f64), BuyError> {
        let (count, cost) = economy::affordable_levels(
            self.hire_base_cost(data),
            data.config.hire_cost_growth,
            self.run.goblins,
            self.run.gold,
            requested,
        );
        if count == 0 {
            return Err(BuyError::CannotAfford);
        }
        self.run.gold -= cost;
        self.run.goblins += count;
        Ok((count, cost))
    }

    /// Pays the expedition cost and rolls for treasure (GDD §5.3). The cost
    /// scales with treasures already found (#6); a complete set is never
    /// charged, since its outcome is a guaranteed empty result.
    pub fn try_explore(&mut self, data: &GameData) -> Result<ExploreResult, BuyError> {
        let cost = self.explore_cost(data);
        if self.run.gold < cost {
            return Err(BuyError::CannotAfford);
        }
        self.persistent.stats.expeditions_launched += 1.0;
        let chance = self.discovery_chance(data);
        let outcome = exploration::roll_treasure(
            &mut self.persistent.rng,
            &data.treasures,
            &self.persistent.discovered_treasures,
            self.persistent.prestige_count,
            chance,
        );
        // A complete hoard rolls `AllDiscovered` before touching the RNG, so
        // don't charge for an expedition that can't find anything — but it still
        // sparks a Hoard Rush so the button stays worth pressing (#7).
        if outcome == ExploreOutcome::AllDiscovered {
            self.trigger_hoard_rush(data.config.hoard_rush_seconds);
            return Ok(ExploreResult::AllDiscovered);
        }
        self.run.gold -= cost;

        Ok(match outcome {
            ExploreOutcome::NothingFound => {
                // A miss is no longer a dead loss: it stirs a Hoard Rush (#7).
                self.trigger_hoard_rush(data.config.hoard_rush_seconds);
                ExploreResult::NothingFound
            }
            ExploreOutcome::AllDiscovered => ExploreResult::AllDiscovered,
            ExploreOutcome::Found(def) => {
                self.persistent.discovered_treasures.push(def.id.clone());
                // A find is a moment worth marking: it ignites a Hoard Rush that
                // lasts longer the rarer the haul, so exploration's best outcome
                // is also its most exciting, and rarity is felt right now.
                let rush = self.treasure_find_rush_seconds(data, def.rarity);
                self.trigger_hoard_rush(rush);
                ExploreResult::Found {
                    name: def.name.clone(),
                    rarity: def.rarity.label(),
                    rush_seconds: rush,
                }
            }
        })
    }

    /// Whether a run upgrade line is unlocked at the current prestige (#10).
    pub fn upgrade_unlocked(&self, def: &crate::data::UpgradeDef) -> bool {
        self.persistent.prestige_count >= def.prestige_required
    }

    /// Buys up to `requested` levels of a run upgrade, capped by its max level
    /// and by affordable gold. Returns `(bought, new_level)`.
    pub fn try_buy_upgrade_bulk(
        &mut self,
        data: &GameData,
        id: &str,
        requested: u32,
    ) -> Result<(u32, u32), BuyError> {
        let def = data.upgrade(id).ok_or(BuyError::UnknownId)?;
        if !self.upgrade_unlocked(def) {
            return Err(BuyError::CannotAfford);
        }
        let level = self.run.upgrade_levels.get(id).copied().unwrap_or(0);
        if level >= def.max_level {
            return Err(BuyError::MaxLevel);
        }
        let remaining = def.max_level - level;
        let (count, cost) = economy::affordable_levels(
            def.base_cost,
            def.cost_growth,
            level,
            self.run.gold,
            requested.min(remaining),
        );
        if count == 0 {
            return Err(BuyError::CannotAfford);
        }
        self.run.gold -= cost;
        self.run.upgrade_levels.insert(id.to_owned(), level + count);
        self.persistent.stats.upgrades_purchased += f64::from(count);
        Ok((count, level + count))
    }

    /// Buys one level of a permanent prestige upgrade with Hoard Points.
    pub fn try_buy_prestige_upgrade(&mut self, data: &GameData, id: &str) -> Result<u32, BuyError> {
        let def = data.prestige_upgrade(id).ok_or(BuyError::UnknownId)?;
        let level = self
            .persistent
            .prestige_upgrade_levels
            .get(id)
            .copied()
            .unwrap_or(0);
        if level >= def.max_level {
            return Err(BuyError::MaxLevel);
        }
        let cost = economy::upgrade_cost(def.base_cost, def.cost_growth, level);
        if self.persistent.hoard_points < cost {
            return Err(BuyError::CannotAfford);
        }
        self.persistent.hoard_points -= cost;
        self.persistent
            .prestige_upgrade_levels
            .insert(id.to_owned(), level + 1);
        Ok(level + 1)
    }

    /// Gold needed for the next prestige, rising with each one done (GDD §12 Q2).
    pub fn prestige_threshold(&self, data: &GameData) -> f64 {
        prestige::current_threshold(&data.config, self.persistent.prestige_count)
    }

    pub fn can_prestige(&self, data: &GameData) -> bool {
        prestige::can_prestige(&data.config, self.run.gold, self.persistent.prestige_count)
    }

    /// Hoard Points the current hoard would grant right now.
    pub fn prestige_preview(&self, data: &GameData) -> f64 {
        prestige::hoard_points_gained(
            &data.config,
            self.run.gold,
            &self.rates(data),
            &self.percents(data),
        )
    }

    /// Converts the hoard into Hoard Points and resets the run (GDD §5.4).
    /// Treasures, achievements, codex, and the prestige tree all persist.
    pub fn try_prestige(&mut self, data: &GameData) -> Option<f64> {
        if !self.can_prestige(data) {
            return None;
        }
        let gained = self.prestige_preview(data);
        self.persistent.hoard_points += gained;
        self.persistent.prestige_count += 1;
        self.run = RunState::new();
        Some(gained)
    }

    // --- unlock conditions ----------------------------------------------

    /// Current value for a condition stat (GDD §5.6). Lifetime stats survive
    /// prestige; `Goblins` is deliberately the current-run count (all tiers).
    pub fn stat_value(&self, key: StatKey) -> f64 {
        match key {
            StatKey::ClicksTotal => self.persistent.stats.clicks_total,
            StatKey::GoldTotalEarned => self.persistent.stats.gold_total_earned,
            StatKey::Goblins => f64::from(self.total_minions()),
            StatKey::TreasuresDiscovered => self.persistent.discovered_treasures.len() as f64,
            StatKey::UpgradesPurchased => self.persistent.stats.upgrades_purchased,
            StatKey::PrestigeCount => f64::from(self.persistent.prestige_count),
            StatKey::AchievementsUnlocked => self.persistent.achievements.progress().0 as f64,
            StatKey::ExpeditionsLaunched => self.persistent.stats.expeditions_launched,
            StatKey::GoldenHoardsCollected => self.persistent.stats.golden_hoards_collected,
        }
    }

    /// Checks all achievement and codex conditions, applying rewards.
    /// Loops until stable so meta-achievements ("unlock 9 others") can chain
    /// within a single frame.
    pub fn check_unlocks(&mut self, data: &GameData) -> Vec<UnlockEvent> {
        let mut events = Vec::new();
        loop {
            let mut changed = false;

            for def in &data.achievements {
                if self.persistent.achievements.is_unlocked(&def.id) {
                    continue;
                }
                if self.stat_value(def.condition.stat) >= def.condition.gte
                    && self.persistent.achievements.unlock(&def.id)
                {
                    self.earn(def.reward.gold);
                    self.persistent.hoard_points += def.reward.hoard_points;
                    events.push(UnlockEvent::Achievement {
                        name: def.name.clone(),
                    });
                    changed = true;
                }
            }

            for def in &data.dragons {
                if self.persistent.unlocked_dragons.contains(&def.id) {
                    continue;
                }
                if self.stat_value(def.unlock.stat) >= def.unlock.gte {
                    self.persistent.unlocked_dragons.push(def.id.clone());
                    events.push(UnlockEvent::Dragon {
                        name: def.name.clone(),
                    });
                    changed = true;
                }
            }

            if !changed {
                return events;
            }
        }
    }
}
