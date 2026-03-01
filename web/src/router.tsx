import { lazy } from 'react';
import { Route, Routes } from 'react-router-dom';

const Dashboard = lazy(() => import('./pages/Dashboard'));
const Projects = lazy(() => import('./pages/Projects'));
const ProjectDetail = lazy(() => import('./pages/ProjectDetail'));
const Sessions = lazy(() => import('./pages/Sessions'));
const SessionDetail = lazy(() => import('./pages/SessionDetail'));
const Prompts = lazy(() => import('./pages/Prompts'));
const Activity = lazy(() => import('./pages/Activity'));
const Agents = lazy(() => import('./pages/Agents'));
const AgentDetailPage = lazy(() => import('./pages/AgentDetail'));
const Skills = lazy(() => import('./pages/Skills'));
const SkillDetailPage = lazy(() => import('./pages/SkillDetail'));

export function AppRoutes() {
  return (
    <Routes>
      <Route path="/" element={<Dashboard />} />
      <Route path="/projects" element={<Projects />} />
      <Route path="/projects/:name" element={<ProjectDetail />} />
      <Route path="/sessions" element={<Sessions />} />
      <Route path="/sessions/:id" element={<SessionDetail />} />
      <Route path="/prompts" element={<Prompts />} />
      <Route path="/activity" element={<Activity />} />
      <Route path="/agents" element={<Agents />} />
      <Route path="/agents/:name" element={<AgentDetailPage />} />
      <Route path="/skills" element={<Skills />} />
      <Route path="/skills/:name" element={<SkillDetailPage />} />
    </Routes>
  );
}
