// In ai/vibe.rs

use crate::{AnalysisResult, ErrorInfo};

#[derive(Debug, Clone)]
pub struct VibeMode {
    pub learning_focus: LearningFocus,
    pub teaching_style: TeachingStyle,
    pub skill_level: SkillLevel,
    pub include_deep_dive: bool,
}

#[derive(Debug, Clone)]
pub enum LearningFocus {
    ConceptualUnderstanding, // Why does this happen?
    PracticalPatterns,       // What patterns prevent this?
    DebuggingSkills,         // How to diagnose similar issues?
    BestPractices,           // Industry standards & conventions
    LanguageFundamentals,    // Core language concepts
    SystemDesign,            // Architecture-level insights
}

#[derive(Debug, Clone)]
pub enum TeachingStyle {
    SocraticMethod,   // Questions that guide discovery
    BuildingBlocks,   // Start simple, build complexity
    RealWorldContext, // Industry examples & war stories
    CompareContrast,  // Show good vs bad approaches
    HandsOnPractice,  // Exercises & challenges
}

#[derive(Debug, Clone)]
pub enum SkillLevel {
    Beginner,     // New to programming/language
    Intermediate, // Knows basics, learning advanced concepts
    Advanced,     // Experienced, wants deep insights
    Expert,       // Teaching-level understanding
}

pub fn generate_teaching_prompt(
    errors: &[ErrorInfo],
    analysis: &AnalysisResult,
    vibe: &VibeMode,
) -> String {
    let mut prompt = String::new();

    // Teaching-focused system message
    prompt.push_str(&get_teaching_system_message(vibe));

    // Error context with learning opportunities
    prompt.push_str("\n## LEARNING OPPORTUNITY\n");
    prompt.push_str(&format!("Error Summary: {}\n", analysis.summary));

    if let Some(first_error) = errors.first() {
        prompt.push_str(&format!("Language: {}\n", first_error.language));
        if let Some(location) = &first_error.location {
            prompt.push_str(&format!("Location: {location}\n"));
        }
    }

    // Frame the learning objectives
    prompt.push_str(&get_learning_objectives(analysis, vibe));

    // Teaching-specific instructions
    prompt.push_str(&get_teaching_instructions(vibe));

    prompt
}

fn get_teaching_system_message(vibe: &VibeMode) -> String {
    let base_teacher = match vibe.teaching_style {
        TeachingStyle::SocraticMethod => {
            "You are a programming mentor who teaches through guided questions. Instead of just giving answers, you ask probing questions that help the student discover the solution themselves."
        },
        TeachingStyle::BuildingBlocks => {
            "You are a programming educator who builds understanding from first principles. You start with the simplest concepts and gradually build complexity."
        },
        TeachingStyle::RealWorldContext => {
            "You are a senior developer who teaches through stories and real-world examples. You share war stories, industry practices, and practical wisdom."
        },
        TeachingStyle::CompareContrast => {
            "You are a programming instructor who teaches by showing contrasts. You demonstrate both good and bad approaches, explaining why each works or fails."
        },
        TeachingStyle::HandsOnPractice => {
            "You are a coding bootcamp instructor who believes in learning by doing. You provide exercises, challenges, and hands-on practice opportunities."
        },
    };

    let skill_adaptation = match vibe.skill_level {
        SkillLevel::Beginner => " Assume the student is new to programming. Use simple language and explain fundamental concepts.",
        SkillLevel::Intermediate => " The student knows basic programming but is learning advanced concepts. Build on their existing knowledge.",
        SkillLevel::Advanced => " The student is experienced. Focus on nuanced insights, edge cases, and architectural considerations.",
        SkillLevel::Expert => " The student has deep expertise. Discuss advanced patterns, performance implications, and design trade-offs.",
    };

    format!("{base_teacher}{skill_adaptation}")
}

fn get_learning_objectives(_analysis: &AnalysisResult, vibe: &VibeMode) -> String {
    let mut objectives = String::from("\n## LEARNING OBJECTIVES\n");
    objectives.push_str("After working through this error, the student should understand:\n");

    match vibe.learning_focus {
        LearningFocus::ConceptualUnderstanding => {
            objectives.push_str("1. The underlying computer science concepts at play\n");
            objectives.push_str("2. Why this error occurs at a fundamental level\n");
            objectives
                .push_str("3. The mental model needed to reason about this class of problems\n");
        }
        LearningFocus::PracticalPatterns => {
            objectives.push_str("1. Common patterns that prevent this type of error\n");
            objectives.push_str("2. Code structures and idioms that lead to robust solutions\n");
            objectives.push_str("3. When and how to apply these patterns in their own code\n");
        }
        LearningFocus::DebuggingSkills => {
            objectives.push_str("1. How to systematically diagnose similar errors\n");
            objectives.push_str("2. Tools and techniques for effective debugging\n");
            objectives.push_str("3. Reading and interpreting error messages and stack traces\n");
        }
        LearningFocus::BestPractices => {
            objectives.push_str("1. Industry-standard approaches to this problem domain\n");
            objectives.push_str("2. Code quality principles that prevent these issues\n");
            objectives.push_str("3. Team practices and conventions around error handling\n");
        }
        LearningFocus::LanguageFundamentals => {
            objectives.push_str("1. Core language features and how they work internally\n");
            objectives.push_str("2. Language-specific idioms and conventions\n");
            objectives.push_str("3. How this error relates to the language's design philosophy\n");
        }
        LearningFocus::SystemDesign => {
            objectives
                .push_str("1. How this error fits into larger system architecture concerns\n");
            objectives.push_str("2. Design patterns and principles that prevent such issues\n");
            objectives.push_str("3. Trade-offs between different architectural approaches\n");
        }
    }

    objectives
}

fn get_teaching_instructions(vibe: &VibeMode) -> String {
    let mut instructions = String::from("\n## TEACHING APPROACH\n");

    match vibe.teaching_style {
        TeachingStyle::SocraticMethod => {
            instructions
                .push_str("Guide the student through discovery by asking questions like:\n");
            instructions.push_str("- What do you think this error message is telling us?\n");
            instructions.push_str("- What conditions might lead to this situation?\n");
            instructions.push_str("- How could we verify our hypothesis about the cause?\n");
            instructions.push_str("- What would happen if we tried approach X vs Y?\n");
            instructions.push_str("\nProvide hints and gentle corrections, but let them reason through the solution.\n");
        }
        TeachingStyle::BuildingBlocks => {
            instructions.push_str("Start with fundamentals and build up:\n");
            instructions.push_str("1. Explain the basic concept that's violated\n");
            instructions.push_str("2. Show a minimal example that demonstrates the principle\n");
            instructions.push_str("3. Gradually add complexity until we reach the full solution\n");
            instructions.push_str("4. Connect each step to the larger understanding\n");
        }
        TeachingStyle::RealWorldContext => {
            instructions.push_str("Share practical wisdom:\n");
            instructions.push_str("- Tell a story about when you or your team hit this issue\n");
            instructions.push_str("- Explain how this error manifests in production systems\n");
            instructions.push_str("- Discuss industry practices around preventing this\n");
            instructions.push_str("- Share tools and techniques professionals actually use\n");
        }
        TeachingStyle::CompareContrast => {
            instructions.push_str("Show the contrast clearly:\n");
            instructions.push_str("❌ BAD: Show the problematic code and explain why it fails\n");
            instructions.push_str("✅ GOOD: Show the corrected version and explain why it works\n");
            instructions.push_str("🤔 ALTERNATIVE: Show other approaches and their trade-offs\n");
            instructions.push_str("⚡ EXPERT: Show advanced techniques and when to use them\n");
        }
        TeachingStyle::HandsOnPractice => {
            instructions.push_str("Provide actionable exercises:\n");
            instructions.push_str("1. A quick fix they can implement immediately\n");
            instructions.push_str("2. A small exercise to practice the concept\n");
            instructions.push_str("3. A challenge problem that extends their understanding\n");
            instructions.push_str("4. Resources for further practice\n");
        }
    }

    if vibe.include_deep_dive {
        instructions.push_str("\n## DEEP DIVE SECTION\n");
        instructions.push_str("Include an advanced section covering:\n");
        instructions.push_str("- Performance implications\n");
        instructions.push_str("- Edge cases and gotchas\n");
        instructions.push_str("- Historical context or language evolution\n");
        instructions.push_str("- Connections to other programming concepts\n");
        instructions.push_str("- Industry trends and future directions\n");
    }

    instructions.push_str("\n## DELIVERABLES\n");
    instructions.push_str("Provide:\n");
    instructions.push_str("1. The immediate fix with clear explanation\n");
    instructions.push_str("2. The educational content tailored to the teaching style\n");
    instructions.push_str("3. Practice exercises or follow-up questions\n");
    instructions.push_str("4. Resources for deeper learning\n");

    instructions
}

pub fn handle_vibe_mode(
    errors: &[ErrorInfo],
    analysis: &AnalysisResult,
    vibe_args: &str,
) -> String {
    let vibe = parse_vibe_args(vibe_args);
    let prompt = generate_teaching_prompt(errors, analysis, &vibe);

    format!(
        "🌊 VIBE MODE: LEARNING-OPTIMIZED PROMPT 🌊\n\n\
        === TEACHING CONFIGURATION ===\n\
        Focus: {:?}\n\
        Style: {:?}\n\
        Level: {:?}\n\
        Deep Dive: {}\n\n\
        === COPY THIS EDUCATIONAL PROMPT ===\n\n\
        {}\n\n\
        === END PROMPT ===\n\n\
        🎓 This prompt will teach programming concepts while fixing your error!\n\
        🚀 Perfect for ChatGPT/Claude when you want to learn, not just fix!\n\
        💡 The AI will explain WHY things work, not just HOW to fix them!",
        vibe.learning_focus,
        vibe.teaching_style,
        vibe.skill_level,
        if vibe.include_deep_dive {
            "Enabled 🔥"
        } else {
            "Disabled"
        },
        prompt
    )
}

fn parse_vibe_args(args: &str) -> VibeMode {
    let mut vibe = VibeMode {
        learning_focus: LearningFocus::ConceptualUnderstanding,
        teaching_style: TeachingStyle::BuildingBlocks,
        skill_level: SkillLevel::Intermediate,
        include_deep_dive: false,
    };

    // Learning focus
    if args.contains("--concepts") {
        vibe.learning_focus = LearningFocus::ConceptualUnderstanding;
    } else if args.contains("--patterns") {
        vibe.learning_focus = LearningFocus::PracticalPatterns;
    } else if args.contains("--debugging") {
        vibe.learning_focus = LearningFocus::DebuggingSkills;
    } else if args.contains("--practices") {
        vibe.learning_focus = LearningFocus::BestPractices;
    } else if args.contains("--fundamentals") {
        vibe.learning_focus = LearningFocus::LanguageFundamentals;
    } else if args.contains("--design") {
        vibe.learning_focus = LearningFocus::SystemDesign;
    }

    // Teaching style
    if args.contains("--socratic") {
        vibe.teaching_style = TeachingStyle::SocraticMethod;
    } else if args.contains("--building-blocks") {
        vibe.teaching_style = TeachingStyle::BuildingBlocks;
    } else if args.contains("--stories") {
        vibe.teaching_style = TeachingStyle::RealWorldContext;
    } else if args.contains("--compare") {
        vibe.teaching_style = TeachingStyle::CompareContrast;
    } else if args.contains("--practice") {
        vibe.teaching_style = TeachingStyle::HandsOnPractice;
    }

    // Skill level
    if args.contains("--beginner") {
        vibe.skill_level = SkillLevel::Beginner;
    } else if args.contains("--intermediate") {
        vibe.skill_level = SkillLevel::Intermediate;
    } else if args.contains("--advanced") {
        vibe.skill_level = SkillLevel::Advanced;
    } else if args.contains("--expert") {
        vibe.skill_level = SkillLevel::Expert;
    }

    // Options
    if args.contains("--deep-dive") {
        vibe.include_deep_dive = true;
    }

    vibe
}

pub const VIBE_HELP: &str = r#"
🌊 VIBE MODE - Generate learning-optimized prompts for AI coders

Turn error fixing into a programming lesson! 

USAGE:
  smels analyze --vibe [FOCUS] [STYLE] [LEVEL] [OPTIONS] -f error.log

LEARNING FOCUS:
  --concepts      Understand WHY this happens (default)
  --patterns      Learn common patterns & idioms  
  --debugging     Master debugging techniques
  --practices     Industry best practices
  --fundamentals  Core language concepts
  --design        System design implications

TEACHING STYLES:
  --socratic         Guide through questions (Socratic method)
  --building-blocks  Start simple, build complexity (default)
  --stories          Real-world examples & war stories
  --compare          Show good vs bad approaches
  --practice         Hands-on exercises & challenges

SKILL LEVELS:
  --beginner      New to programming/language
  --intermediate  Know basics, learning advanced (default)
  --advanced      Experienced developer
  --expert        Teaching-level understanding

OPTIONS:
  --deep-dive     Include advanced concepts & edge cases

EXAMPLES:
  # Learn Rust concepts through Socratic questioning
  smels analyze --vibe --concepts --socratic --intermediate -f panic.log

  # Get practical patterns with real-world stories
  smels analyze --vibe --patterns --stories --advanced -f error.txt

  # Beginner-friendly step-by-step debugging lesson
  smels analyze --vibe --debugging --building-blocks --beginner -f build.log

  # Expert-level system design insights with deep dive
  smels analyze --vibe --design --compare --expert --deep-dive -f crash.log
"#;
